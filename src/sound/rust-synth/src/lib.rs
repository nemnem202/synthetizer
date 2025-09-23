use js_sys::{Array, Atomics, Float32Array, Int32Array, SharedArrayBuffer, Uint8Array};
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;
use std::cell::RefCell;
use std::f32::consts::TAU;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use web_sys::console;

// =============================================================================
// CONSTANTES
// =============================================================================

const FLAG_INDEX: u32 = 0;
const READ_INDEX: u32 = 1;
const WRITE_INDEX: u32 = 2;
const HEADERS_SIZE_BYTES: u32 = 3 * 4;

const MIDI_EVENT_SIZE: u32 = 4;
const MIDI_QUEUE_CAPACITY: u32 = 64;
const MIDI_WRITE_INDEX: u32 = 0;
const MIDI_READ_INDEX: u32 = 1;

const SAMPLE_RATE: f32 = 44100.0;
const FREQ_A4: f32 = 440.0;

const OSC_QUEUE_CAPACITY: u32 = 100;

// =============================================================================
// TYPES ET ENUMS
// =============================================================================

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventType {
    NoteOff = 0,
    NoteOn = 1,
}

impl TryFrom<u8> for EventType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EventType::NoteOff),
            1 => Ok(EventType::NoteOn),
            _ => Err("Valeur d'événement MIDI inconnue"),
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum WaveType {
    Sine,
    Square,
    Saw,
    Triangle,
}

impl TryFrom<u8> for WaveType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(WaveType::Sine),
            1 => Ok(WaveType::Square),
            2 => Ok(WaveType::Saw),
            3 => Ok(WaveType::Triangle),
            _ => Err("Valeur de WaveType invalide"),
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct NoteDTO {
    pub value: u8,
    pub velocity: u8,
}

// =============================================================================
// GÉNÉRATEUR D'ONDES
// =============================================================================

pub struct WaveGenerator;

impl WaveGenerator {
    pub fn midi_to_freq(note: u8) -> f32 {
        FREQ_A4 * 2.0f32.powf((note as f32 - 69.0) / 12.0)
    }

    pub fn generate_sample(phase: f32, wave_type: WaveType) -> f32 {
        match wave_type {
            WaveType::Sine => (phase * TAU).sin(),
            WaveType::Square => {
                if phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            WaveType::Saw => 2.0 * (phase - 0.5),
            WaveType::Triangle => {
                let v = if phase < 0.5 {
                    4.0 * phase - 1.0
                } else {
                    3.0 - 4.0 * phase
                };
                v
            }
        }
    }
}

// =============================================================================
// OSCILLATEUR ET ADSR
// =============================================================================

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct Oscillator {
    id: u8,
    wave_type: WaveType,
    attack_length: u64,
    decay_length: u64,
    sustain_gain: f32,
    release_length: u64,
    frequency_shift: f32,
    phase_shift: f32,
    delay_length: u64,
    gain: f32,
    gainL: f32,
    gainR: f32,
}

impl Oscillator {
    pub fn apply_adsr(&self, state: &mut NoteOscState, note_has_ended: bool, value: &mut f32) {
        if note_has_ended {
            if state.end_sample_index >= self.release_length {
                state.finished = true;
                *value = 0.0;
                return;
            }

            *value *= ((self.release_length as f32 - state.end_sample_index as f32)
                / self.release_length as f32);
        }

        if state.start_sample_index <= self.delay_length {
            *value = 0.0
        } else if state.start_sample_index <= self.attack_length + self.delay_length {
            *value *= (state.start_sample_index as f32 - self.delay_length as f32)
                / self.attack_length as f32;
        } else if state.start_sample_index
            <= self.attack_length + self.decay_length + self.delay_length
        {
            *value *= 1.0
                + ((state.start_sample_index as f32
                    - self.attack_length as f32
                    - self.delay_length as f32)
                    * (self.sustain_gain - 1.0)
                    / self.decay_length as f32);
        } else {
            *value *= self.sustain_gain;
        }
    }

    pub fn generate_sample(
        &self,
        note_value: u8,
        note_velocity: u8,
        state: &mut NoteOscState,
        note_has_ended: bool,
    ) -> (f32, f32) {
        if state.finished {
            return (0.0, 0.0);
        }

        let freq: f32 = WaveGenerator::midi_to_freq(note_value) * self.frequency_shift;

        let mut value = WaveGenerator::generate_sample(state.current_phase, self.wave_type)
            * note_velocity as f32
            * self.gain
            / 127.0;

        self.apply_adsr(state, note_has_ended, &mut value);

        // Mise à jour de l'état
        state.current_phase += freq / SAMPLE_RATE;
        state.current_phase %= 1.0;
        state.start_sample_index += 1;

        if note_has_ended {
            state.end_sample_index += 1;
        }

        let left = value * self.gainL;
        let right = value * self.gainR;

        (left, right)
    }
}

// =============================================================================
// Effets DSP
// =============================================================================
pub struct BiquadCoeffs {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
}

impl BiquadCoeffs {
    pub fn calc_biquad_coeffs(frequency: f32, q: f32, sample_rate: f32) -> BiquadCoeffs {
        let w0 = 2.0 * std::f32::consts::PI * frequency / sample_rate;
        let alpha = (w0).sin() / (2.0 * q);

        let b0 = (1.0 - w0.cos()) / 2.0;
        let b1 = 1.0 - w0.cos();
        let b2 = (1.0 - w0.cos()) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * w0.cos();
        let a2 = 1.0 - alpha;

        // Normalisation
        BiquadCoeffs {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }
}
struct BiquadFilter {
    coeffs: BiquadCoeffs,
    z1l: f32,
    z1r: f32,
    z2l: f32,
    z2r: f32,
}

impl BiquadFilter {
    pub fn process(&mut self, input_sample_r: f32, input_sample_l: f32) -> (f32, f32) {
        let output_sample_r = self.coeffs.b0 * input_sample_r + self.z1r;
        let output_sample_l = self.coeffs.b0 * input_sample_l + self.z1l;

        self.z1l = self.coeffs.b1 * input_sample_l - self.coeffs.a1 * output_sample_l + self.z2l;
        self.z1r = self.coeffs.b1 * input_sample_r - self.coeffs.a1 * output_sample_r + self.z2r;

        self.z2l = self.coeffs.b2 * input_sample_l - self.coeffs.a2 * output_sample_l;
        self.z2r = self.coeffs.b2 * input_sample_r - self.coeffs.a2 * output_sample_r;

        (output_sample_r, output_sample_l)
    }
}

struct Echo {
    pub delay: usize,
    pub feedback: f32,
    pub memory: MemoryBuffer,
}

impl Echo {
    pub fn new(delay: usize, feedback: f32) -> Self {
        Echo {
            delay: delay,
            feedback: feedback.max(0.0).min(1.0),
            memory: MemoryBuffer::new(44100, 10.0),
        }
    }

    pub fn process(&mut self, input_l: &mut f32, input_r: &mut f32) {
        let (l, r) = self.memory.read(self.delay);
        *input_l += l * self.feedback;
        *input_r += r * self.feedback;
        self.memory.write(*input_l, *input_r);
    }
}

// =============================================================================
// ÉTAT D'OSCILLATEUR ET NOTE
// =============================================================================

#[derive(Debug, Clone)]
pub struct NoteOscState {
    pub current_phase: f32,
    pub start_sample_index: u64,
    pub end_sample_index: u64,
    pub finished: bool,
}

impl NoteOscState {
    pub fn new(phase_shift: f32) -> Self {
        Self {
            current_phase: phase_shift % 1.0,
            start_sample_index: 0,
            end_sample_index: 0,
            finished: false,
        }
    }

    pub fn reset(&mut self, phase_shift: f32) {
        self.current_phase = phase_shift % 1.0;
        self.start_sample_index = 0;
        self.end_sample_index = 0;
        self.finished = false;
    }
}

#[derive(Debug, Clone)]
pub struct Note {
    pub value: u8,
    pub velocity: u8,
    pub has_ended: bool,
    pub to_remove: bool,
    pub start_sample_index: u64,
    pub end_sample_index: u64,
    pub current_phase: f32,
    pub osc_states: Vec<NoteOscState>,
}

impl Note {
    pub fn new(value: u8, velocity: u8, oscillators: &[Oscillator]) -> Self {
        let osc_states = oscillators
            .iter()
            .map(|osc| NoteOscState::new(osc.phase_shift))
            .collect();

        Note {
            value,
            velocity,
            has_ended: false,
            to_remove: false,
            start_sample_index: 0,
            end_sample_index: 0,
            current_phase: 0.0,
            osc_states,
        }
    }

    pub fn restart(&mut self, oscillators: &[Oscillator]) {
        self.has_ended = false;
        self.end_sample_index = 0;
        self.start_sample_index = 0;

        // Réajuster si le nombre d'oscillateurs a changé
        if self.osc_states.len() != oscillators.len() {
            self.osc_states = oscillators
                .iter()
                .map(|osc| NoteOscState::new(osc.phase_shift))
                .collect();
        } else {
            for (state, osc) in self.osc_states.iter_mut().zip(oscillators.iter()) {
                state.reset(osc.phase_shift);
            }
        }
    }

    pub fn end_note(&mut self) {
        self.has_ended = true;
    }

    pub fn is_finished(&self) -> bool {
        self.osc_states.iter().all(|s| s.finished)
    }

    pub fn generate_sample(&mut self, oscillators: &[Oscillator]) -> (f32, f32) {
        if self.to_remove {
            return (0.0, 0.0);
        }

        let mut note_sum_l = 0.0;
        let mut note_sum_r = 0.0;

        for (osc_index, oscillator) in oscillators.iter().enumerate() {
            if let Some(state) = self.osc_states.get_mut(osc_index) {
                let (l, r) =
                    oscillator.generate_sample(self.value, self.velocity, state, self.has_ended);
                note_sum_l += l;
                note_sum_r += r;
            }
        }

        (note_sum_l, note_sum_r)
    }
}

// =============================================================================
// GESTIONNAIRE DE NOTES
// =============================================================================

pub struct NoteManager {
    notes: Vec<Note>,
}

impl NoteManager {
    pub fn new() -> Self {
        Self { notes: Vec::new() }
    }

    pub fn add_note(&mut self, dto: &NoteDTO, oscillators: &[Oscillator]) {
        if let Some(existing_note) = self.notes.iter_mut().find(|n| n.value == dto.value) {
            console::log_1(&"La note existe déjà OEOE".into());
            if existing_note.has_ended {
                existing_note.restart(oscillators);
            }
        } else {
            self.notes
                .push(Note::new(dto.value, dto.velocity, oscillators));
        }
    }

    pub fn end_note(&mut self, dto: &NoteDTO) {
        for note in self.notes.iter_mut() {
            if note.value == dto.value && !note.has_ended {
                note.end_note();
            }
        }
    }

    pub fn cleanup_finished_notes(&mut self) {
        let initial_count = self.notes.len();
        self.notes.retain(|note| {
            let finished = note.is_finished();

            !finished
        });
    }

    pub fn generate_samples(&mut self, sample_count: i32, oscillators: &[Oscillator]) -> Vec<f32> {
        // sample_count inclut déjà les 2 canaux
        let mut samples = Vec::with_capacity(sample_count as usize);

        // nb de frames stéréo = moitié de sample_count
        let frame_count = sample_count / 2;

        for _ in 0..frame_count {
            let mut mixed_l = 0.0;
            let mut mixed_r = 0.0;

            if self.notes.is_empty() {
            } else {
                for note in self.notes.iter_mut() {
                    let (l, r) = note.generate_sample(oscillators);
                    mixed_l += l;
                    mixed_r += r;
                }

                if !oscillators.is_empty() {
                    mixed_l /= oscillators.len() as f32;
                    mixed_r /= oscillators.len() as f32;
                }

                TEST_BIQUAD.with(|biquad| {
                    let mut biquad = biquad.lock().unwrap(); // Mutex guard

                    let (l_out, r_out) = biquad.process(mixed_l, mixed_r);
                    mixed_l = l_out;
                    mixed_r = r_out;
                });
            }

            TEST_DELAY.with(|ech| {
                let mut echo = ech.lock().unwrap();
                echo.process(&mut mixed_l, &mut mixed_r);
            });

            mixed_l *= 0.1;
            mixed_r *= 0.1;

            samples.push(mixed_l);
            samples.push(mixed_r);

            continue;
        }

        self.cleanup_finished_notes();
        samples
    }
}

// =============================================================================
// STRUCTURES DE BUFFERS
// =============================================================================

pub struct AudioBuffers {
    flag: Int32Array,
    read_idx: Int32Array,
    write_idx: Int32Array,
    ring_buffer: Float32Array,
}

pub struct MidiBuffers {
    write_idx: Int32Array,
    read_idx: Int32Array,
    queue: Uint8Array,
}

impl MidiBuffers {
    pub fn dequeue_event(&self) -> Option<NoteDTO> {
        let read_pos = Atomics::load(&self.read_idx, 0).unwrap() as u32;
        let write_pos = Atomics::load(&self.write_idx, 0).unwrap() as u32;

        if read_pos == write_pos {
            return None;
        }

        let event_offset = read_pos * MIDI_EVENT_SIZE;
        let _event_type = self.queue.get_index(event_offset);
        let note_value = self.queue.get_index(event_offset + 1);
        let velocity = self.queue.get_index(event_offset + 2);

        let new_read_pos = (read_pos + 1) % MIDI_QUEUE_CAPACITY;
        Atomics::store(&self.read_idx, 0, new_read_pos as i32).unwrap();

        Some(NoteDTO {
            value: note_value,
            velocity,
        })
    }

    pub fn process_all_events<F>(&self, mut handler: F) -> u32
    where
        F: FnMut(&NoteDTO),
    {
        let mut events_processed = 0;

        while let Some(dto) = self.dequeue_event() {
            events_processed += 1;
            handler(&dto);
        }

        events_processed
    }
}

pub struct OscillatorBuffers {
    write_idx: Int32Array,
    read_idx: Int32Array,
    queue: Uint8Array, // 8 octets par événement
}

pub struct SharedBuffers {
    audio: AudioBuffers,
    midi: MidiBuffers,
    osc: OscillatorBuffers,
}

pub struct MemoryBuffer {
    buffer: Vec<f32>,
    size: usize,
    write_index: usize,
}

impl MemoryBuffer {
    /// Crée un buffer pour `duration_seconds` à `sample_rate` Hz
    pub fn new(sample_rate: usize, duration_seconds: f32) -> Self {
        let size = (sample_rate as f32 * duration_seconds * 2.0) as usize;
        Self {
            buffer: vec![0.0; size],
            size,
            write_index: 0,
        }
    }

    /// Écrit un sample dans le buffer
    pub fn write(&mut self, sample_l: f32, sample_r: f32) {
        self.buffer[self.write_index] = sample_l;
        self.buffer[self.write_index + 1] = sample_r;
        self.write_index = (self.write_index + 2) % self.size;
    }

    /// Lit un sample avec un délai en nombre d’échantillons
    pub fn read(&self, delay_samples: usize) -> (f32, f32) {
        // on calcule la position de lecture "dans le passé"
        let read_index = (self.size + self.write_index - delay_samples) % self.size;
        (self.buffer[read_index], self.buffer[read_index + 1])
    }
}
// =============================================================================
// GESTIONNAIRE DE RING BUFFER
// =============================================================================

pub struct RingBufferManager<'a> {
    ring_buffer: &'a Float32Array,
    write_idx_atomic: &'a Int32Array,
    buffer_size: i32,
}

impl<'a> RingBufferManager<'a> {
    pub fn new(ring_buffer: &'a Float32Array, write_idx_atomic: &'a Int32Array) -> Self {
        Self {
            ring_buffer,
            write_idx_atomic,
            buffer_size: ring_buffer.length() as i32,
        }
    }

    pub fn write_samples(&self, samples: &[f32]) {
        let space = samples.len() as i32;
        let mut current_write_idx = Atomics::load(self.write_idx_atomic, 0).unwrap();
        let chunk_array = Float32Array::from(samples);

        let contiguous_space = std::cmp::min(space, self.buffer_size - current_write_idx);

        if contiguous_space > 0 {
            self.ring_buffer.set(
                &chunk_array.subarray(0, contiguous_space as u32),
                current_write_idx as u32,
            );
        }

        if space > contiguous_space {
            let rest = space - contiguous_space;
            self.ring_buffer.set(
                &chunk_array.subarray(contiguous_space as u32, (contiguous_space + rest) as u32),
                0,
            );
            current_write_idx = rest;
        } else {
            current_write_idx = (current_write_idx + contiguous_space) % self.buffer_size;
        }

        Atomics::store(self.write_idx_atomic, 0, current_write_idx).unwrap();
    }
}

// =============================================================================
// PROCESSEUR AUDIO PRINCIPAL
// =============================================================================

pub struct AudioProcessor {
    note_manager: NoteManager,
    oscillators: Vec<Oscillator>,
    global_sample_index: u64,
}

impl AudioProcessor {
    pub fn new() -> Self {
        Self {
            note_manager: NoteManager::new(),
            oscillators: Vec::new(),
            global_sample_index: 0,
        }
    }

    pub fn process_midi_events(&mut self, midi: &MidiBuffers) -> u32 {
        midi.process_all_events(|dto| {
            if dto.velocity > 0 {
                self.note_manager.add_note(dto, &self.oscillators);
            } else {
                self.note_manager.end_note(dto);
            }
        })
    }

    pub fn generate_audio_chunk(&mut self, sample_count: i32) -> Vec<f32> {
        self.global_sample_index += sample_count as u64;
        self.note_manager
            .generate_samples(sample_count, &self.oscillators)
    }

    pub fn fill_audio_buffer(&mut self, space: i32, ring_buffer_manager: &RingBufferManager) {
        let samples = self.generate_audio_chunk(space);
        ring_buffer_manager.write_samples(&samples);
    }

    pub fn process_osc_events(&mut self, osc_buffers: &OscillatorBuffers) {
        let mut read_pos = Atomics::load(&osc_buffers.read_idx, 0).unwrap() as u32;
        let write_pos = Atomics::load(&osc_buffers.write_idx, 0).unwrap() as u32;

        if read_pos == write_pos {
            return;
        };

        while read_pos != write_pos {
            let offset = read_pos * 8; // OSC_EVENT_SIZE = 8
            let event_type = osc_buffers.queue.get_index(offset); // OK
            let osc_index = osc_buffers.queue.get_index(offset + 1); // <-- corrige ici
            let key = osc_buffers.queue.get_index(offset + 2);

            let value = {
                let mut bytes = [0u8; 4];
                for i in 0..4 {
                    bytes[i] = osc_buffers.queue.get_index(offset + 3 + i as u32);
                }
                f32::from_le_bytes(bytes)
            };

            match event_type {
                0 => {
                    // add
                    self.oscillators.push(Oscillator {
                        id: osc_index,
                        wave_type: WaveType::Sine,
                        attack_length: self.convert_ms_to_sample(500.0) as u64,
                        decay_length: self.convert_ms_to_sample(500.0) as u64,
                        sustain_gain: 0.5,
                        release_length: self.convert_ms_to_sample(500.0) as u64,
                        frequency_shift: 1.0,
                        delay_length: self.convert_ms_to_sample(0.0) as u64,
                        phase_shift: 0.0,
                        gain: 0.5,
                        gainL: 1.0,
                        gainR: 1.0,
                    });
                    console::log_1(
                        &format!("Oscillateur créé à l'index {}", self.oscillators.len() - 1)
                            .into(),
                    );
                }
                1 => {
                    // remove
                    if let Some(pos) = self.oscillators.iter().position(|osc| osc.id == osc_index) {
                        self.oscillators.remove(pos);
                        console::log_1(&format!("Oscillateur {} supprimé", osc_index).into());
                    } else {
                        console::log_1(&format!("Oscillateur {} introuvable", osc_index).into());
                    }
                }
                2 => {
                    // update
                    if let Some(osc) = self.oscillators.get_mut(osc_index as usize) {
                        match key {
                            1 => osc.attack_length = value as u64,
                            2 => osc.release_length = value as u64,
                            3 => osc.decay_length = value as u64,
                            4 => osc.sustain_gain = value * 0.1,
                            5 => osc.gain = value * 0.1,
                            6 => osc.delay_length = value as u64,
                            7 => osc.frequency_shift = value,
                            8 => osc.phase_shift = value,
                            9 => {
                                if let Ok(wt) = WaveType::try_from(value as u8) {
                                    osc.wave_type = wt;
                                }
                            }
                            10 => {
                                osc.gainL = (1.0 - value) / 2.0;
                                osc.gainR = (1.0 + value) / 2.0
                            }

                            _ => {}
                        }
                        console::log_1(
                            &format!(
                                "Oscillateur {} mis à jour, key {}, value: {}",
                                osc_index, key, value as u64
                            )
                            .into(),
                        );
                    }
                }
                _ => {}
            }

            read_pos = (read_pos + 1) % OSC_QUEUE_CAPACITY;
        }

        Atomics::store(&osc_buffers.read_idx, 0, read_pos as i32).unwrap();
    }

    pub fn convert_ms_to_sample(&self, ms: f32) -> f32 {
        (ms / 1000.0 * SAMPLE_RATE).floor()
    }
}

// =============================================================================
// VARIABLES GLOBALES ET CONFIGURATIONS
// =============================================================================

thread_local! {
    static SHARED_BUFFERS: OnceCell<SharedBuffers> = OnceCell::new();
    static AUDIO_PROCESSOR: RefCell<Option<AudioProcessor>> = RefCell::new(None);

    static TEST_BIQUAD: Lazy<Mutex<BiquadFilter>> = Lazy::new(|| {
        let coeffs = BiquadCoeffs::calc_biquad_coeffs(800.0, 0.7, SAMPLE_RATE);
        Mutex::new(BiquadFilter {
            coeffs,
            z1l: 0.0,
            z2l: 0.0,
            z1r: 0.0,
            z2r: 0.0,
        })
    });

    // static DELAY_BUFFER: Lazy<Mutex<MemoryBuffer>> = Lazy::new(|| {
    // let  buffer = MemoryBuffer::new(44100, 10.0); // 10s à 44.1kHz
    // Mutex::new(buffer)
    // });

    static TEST_DELAY: Lazy<Mutex<Echo>> = Lazy::new(|| {
        let  echo = Echo::new((44100.0 * 0.3 * 2.0) as usize, 0.5);
        Mutex::new(echo)
    });
}

// =============================================================================
// INITIALISATION ET BOUCLE PRINCIPALE
// =============================================================================

#[wasm_bindgen]

pub fn init_audio_thread(
    shared_audio_buffer: SharedArrayBuffer,
    ring_buffer_size: u32,
    midi_buffer: SharedArrayBuffer,
    osc_buffer: SharedArrayBuffer,
) {
    // -------- Audio --------
    let control_arr = Int32Array::new(&shared_audio_buffer);
    let flag = control_arr.subarray(FLAG_INDEX, FLAG_INDEX + 1);
    let read_idx = control_arr.subarray(READ_INDEX, READ_INDEX + 1);
    let write_idx = control_arr.subarray(WRITE_INDEX, WRITE_INDEX + 1);

    let audio_data_start_elem = (HEADERS_SIZE_BYTES / 4) as u32;
    let ring_buffer_end_elem = audio_data_start_elem + ring_buffer_size;
    let ring_buffer = Float32Array::new(&shared_audio_buffer)
        .subarray(audio_data_start_elem, ring_buffer_end_elem);

    // -------- MIDI --------
    let midi_control_arr = Int32Array::new(&midi_buffer);
    let midi_write_idx = midi_control_arr.subarray(MIDI_WRITE_INDEX, MIDI_WRITE_INDEX + 1);
    let midi_read_idx = midi_control_arr.subarray(MIDI_READ_INDEX, MIDI_READ_INDEX + 1);
    let midi_queue = Uint8Array::new(&midi_buffer).subarray(8, midi_buffer.byte_length());

    // -------- OSCILLATEURS --------
    let osc_control_arr = Int32Array::new(&osc_buffer);
    let osc_write_idx = osc_control_arr.subarray(0, 1);
    let osc_read_idx = osc_control_arr.subarray(1, 2);
    let osc_queue = Uint8Array::new(&osc_buffer).subarray(8, osc_buffer.byte_length());

    // -------- SharedBuffers --------
    let shared_buffers = SharedBuffers {
        audio: AudioBuffers {
            flag,
            read_idx,
            write_idx,
            ring_buffer,
        },
        midi: MidiBuffers {
            write_idx: midi_write_idx,
            read_idx: midi_read_idx,
            queue: midi_queue,
        },
        osc: OscillatorBuffers {
            write_idx: osc_write_idx,
            read_idx: osc_read_idx,
            queue: osc_queue,
        },
    };
    // Initialisation du processeur audio avec les oscillateurs de test
    let audio_processor = AudioProcessor::new();

    SHARED_BUFFERS.with(|c| c.set(shared_buffers));
    AUDIO_PROCESSOR.with(|p| *p.borrow_mut() = Some(audio_processor));

    console::log_1(&"Buffers et processeur audio initialisés".into());
}

#[wasm_bindgen]
pub fn start_audio_processing_loop() {
    SHARED_BUFFERS.with(|cell| {
        let buffers = cell.get().expect("SharedBuffers not initialized!");
        audio_producer_loop(buffers);
    });
}

fn audio_producer_loop(buffers: &SharedBuffers) {
    let flag = &buffers.audio.flag;
    let read_idx = &buffers.audio.read_idx;
    let write_idx = &buffers.audio.write_idx;
    let ring_buffer = &buffers.audio.ring_buffer;
    let midi = &buffers.midi;

    console::log_1(&"Démarrage de la boucle audio (infinie)".into());

    loop {
        Atomics::wait(flag, 0, 1).unwrap();

        AUDIO_PROCESSOR.with(|processor_cell| {
            if let Some(ref mut processor) = *processor_cell.borrow_mut() {
                // Traitement des événements MIDI
                let events_processed = processor.process_midi_events(midi);

                processor.process_osc_events(&buffers.osc);
                // Vérification si la MIDI queue et le tableau de notes sont vides
                // if processor.note_manager.notes.is_empty() && events_processed == 0 {
                //     processor.process_osc_events(&buffers.osc);
                // }

                // Calcul de l'espace disponible dans le ring buffer
                let r_idx = Atomics::load(read_idx, 0).unwrap();
                let w_idx = Atomics::load(write_idx, 0).unwrap();
                let ring_buffer_len = ring_buffer.length() as i32;
                let space_available = (r_idx - w_idx - 1 + ring_buffer_len) % ring_buffer_len;

                if space_available > 0 {
                    let ring_buffer_manager = RingBufferManager::new(ring_buffer, write_idx);
                    processor.fill_audio_buffer(space_available, &ring_buffer_manager);
                }
            }
        });

        Atomics::store(flag, 0, 0).unwrap();
        Atomics::notify(flag, 0).unwrap();
    }
}
