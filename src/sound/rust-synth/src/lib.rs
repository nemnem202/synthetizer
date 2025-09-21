use js_sys::{Atomics, Float32Array, Int32Array, SharedArrayBuffer, Uint8Array};
use once_cell::unsync::OnceCell;
use std::cell::RefCell;
use std::f32::consts::TAU;
use wasm_bindgen::prelude::*;
use web_sys::console;

const FLAG_INDEX: u32 = 0;
const READ_INDEX: u32 = 1;
const WRITE_INDEX: u32 = 2;
const HEADERS_SIZE_BYTES: u32 = 3 * 4;

const MIDI_EVENT_SIZE: u32 = 4;
const MIDI_QUEUE_CAPACITY: u32 = 64;
const MIDI_WRITE_INDEX: u32 = 0;
const MIDI_READ_INDEX: u32 = 1;

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventType {
    NoteOff = 0,
    NoteOn = 1,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum WaveType {
    Sine,
    Square,
    Saw,
    Triangle,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct Oscillator {
    wave_type: WaveType,
    attack_length: u64,
    decay_length: u64,
    sustain_gain: f32,
    release_length: u64,
    frequency_shift: f32,
}

static TEST_OSCILLATOR: Oscillator = Oscillator {
    wave_type: WaveType::Sine,
    attack_length: 100,
    decay_length: 3000,
    sustain_gain: 0.0,
    release_length: 10,
    frequency_shift: 2.0,
};

static TEST_OSCILLATOR_2: Oscillator = Oscillator {
    wave_type: WaveType::Sine,
    attack_length: 50000,
    decay_length: 80000,
    sustain_gain: 0.5,
    release_length: 50000,
    frequency_shift: 1.0,
};

static TEST_OSCILLATOR_3: Oscillator = Oscillator {
    wave_type: WaveType::Sine,
    attack_length: 1000,
    decay_length: 5000,
    sustain_gain: 0.8,
    release_length: 1000,
    frequency_shift: 0.5,
};

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

pub struct SharedBuffers {
    audio: AudioBuffers,
    midi: MidiBuffers,
}

thread_local! {
    static SHARED_BUFFERS: OnceCell<SharedBuffers> = OnceCell::new();
    pub static PLAYED_NOTES: RefCell<Vec<Note>> = RefCell::new(Vec::new());
    pub static OSCILLATORS: RefCell<Vec<Oscillator>> = RefCell::new(vec![TEST_OSCILLATOR, TEST_OSCILLATOR_2, TEST_OSCILLATOR_3]);
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct NoteDTO {
    pub value: u8,
    pub velocity: u8,
}

#[derive(Debug, Clone)]
pub struct NoteOscState {
    pub current_phase: f32,
    pub start_sample_index: u64, // samples depuis le note-on (pour ADSR attack/decay)
    pub end_sample_index: u64,   // samples depuis le note-off (pour release)
    pub finished: bool,          // true quand cet osc a fini son release
}

#[derive(Debug, Clone)]
pub struct Note {
    pub value: u8,
    pub velocity: u8,
    pub has_ended: bool,
    pub to_remove: bool, // on garde pour l'instant, mais on calculera ça à partir des osc_states
    pub start_sample_index: u64,
    pub end_sample_index: u64,
    pub current_phase: f32,
    pub osc_states: Vec<NoteOscState>, // <-- nouvel emplacement des états par osc
}

impl Note {
    // num_osc = nombre d'oscillateurs à l'instant de la création
    fn new(value: u8, velocity: u8, num_osc: usize) -> Self {
        let osc_states = (0..num_osc)
            .map(|_| NoteOscState {
                current_phase: 0.0,
                start_sample_index: 0,
                end_sample_index: 0,
                finished: false,
            })
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
}

fn add_note(dto: &NoteDTO, current_sample_index: u64) {
    console::log_1(
        &format!(
            "Nouvelle note: valeur={}, fréquence={}",
            dto.value,
            midi_to_freq(dto.value)
        )
        .into(),
    );
    PLAYED_NOTES.with(|notes| {
        let mut notes_mut = notes.borrow_mut();

        if let Some(existing_note) = notes_mut.iter_mut().find(|n| n.value == dto.value) {
            console::log_1(&format!("La note existe déja").into());
            if existing_note.has_ended {
                existing_note.has_ended = false;
                existing_note.end_sample_index = 0;
                existing_note.start_sample_index = 0;

                // réinitialise les osc_states : si le nombre d'osc a changé, recrée le vecteur
                OSCILLATORS.with(|osc| {
                    let osc_len = osc.borrow().len();
                    if existing_note.osc_states.len() != osc_len {
                        existing_note.osc_states = (0..osc_len)
                            .map(|_| NoteOscState {
                                current_phase: 0.0,
                                start_sample_index: 0,
                                end_sample_index: 0,
                                finished: false,
                            })
                            .collect();
                    } else {
                        for s in existing_note.osc_states.iter_mut() {
                            s.current_phase = 0.0;
                            s.start_sample_index = 0;
                            s.end_sample_index = 0;
                            s.finished = false;
                        }
                    }
                });
            }
        } else {
            // on récupère le nombre d'oscillateurs courants et on crée la note avec cet espace d'états
            OSCILLATORS.with(|osc| {
                let osc_len = osc.borrow().len();
                notes_mut.push(Note::new(dto.value, dto.velocity, osc_len));
            });
        }
    });
}

#[wasm_bindgen]
pub fn init_audio_thread(
    shared_audio_buffer: SharedArrayBuffer,
    ring_buffer_size: u32,
    midi_buffer: SharedArrayBuffer,
) {
    let control_arr = Int32Array::new(&shared_audio_buffer);
    let flag = control_arr.subarray(FLAG_INDEX, FLAG_INDEX + 1);
    let read_idx = control_arr.subarray(READ_INDEX, READ_INDEX + 1);
    let write_idx = control_arr.subarray(WRITE_INDEX, WRITE_INDEX + 1);

    let audio_data_start_elem = (HEADERS_SIZE_BYTES / 4) as u32;
    let ring_buffer_end_elem = audio_data_start_elem + ring_buffer_size;
    let ring_buffer = Float32Array::new(&shared_audio_buffer)
        .subarray(audio_data_start_elem, ring_buffer_end_elem);

    let midi_control_arr = Int32Array::new(&midi_buffer);
    let midi_write_idx = midi_control_arr.subarray(MIDI_WRITE_INDEX, MIDI_WRITE_INDEX + 1);
    let midi_read_idx = midi_control_arr.subarray(MIDI_READ_INDEX, MIDI_READ_INDEX + 1);

    let midi_queue = Uint8Array::new(&midi_buffer).subarray(8, midi_buffer.byte_length());

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
    };

    SHARED_BUFFERS.with(|c| c.set(shared_buffers));
    console::log_1(&"Buffers initialisés".into());
}

fn dequeue_midi_event(midi: &MidiBuffers) -> Option<NoteDTO> {
    let read_pos = Atomics::load(&midi.read_idx, 0).unwrap() as u32;
    let write_pos = Atomics::load(&midi.write_idx, 0).unwrap() as u32;

    if read_pos == write_pos {
        return None;
    }

    let event_offset = read_pos * MIDI_EVENT_SIZE;
    let event_type = midi.queue.get_index(event_offset);
    let note_value = midi.queue.get_index(event_offset + 1);
    let velocity = midi.queue.get_index(event_offset + 2);

    let new_read_pos = (read_pos + 1) % MIDI_QUEUE_CAPACITY;
    Atomics::store(&midi.read_idx, 0, new_read_pos as i32).unwrap();

    Some(NoteDTO {
        value: note_value,
        velocity,
    })
}

fn process_all_midi_events(midi: &MidiBuffers, current_sample_index: u64) {
    let mut events_processed = 0;

    while let Some(dto) = dequeue_midi_event(midi) {
        events_processed += 1;

        if dto.velocity > 0 {
            add_note(&dto, current_sample_index);
        } else {
            set_note_end(&dto);
        }
    }

    if events_processed > 0 {
        console::log_1(&format!("{} événements MIDI traités", events_processed).into());
    }
}

fn set_note_end(dto: &NoteDTO) {
    PLAYED_NOTES.with(|notes| {
        let mut notes_mut = notes.borrow_mut();
        for note in notes_mut.iter_mut() {
            if note.value == dto.value && !note.has_ended {
                note.has_ended = true;
            }
        }
    });
}

const SAMPLE_RATE: f32 = 44100.0;

const FREQ_A4: f32 = 440.0;

fn midi_to_freq(note: u8) -> f32 {
    FREQ_A4 * 2.0f32.powf((note as f32 - 69.0) / 12.0)
}

fn generate_sample_for_note(note: &mut Note, osc: &Oscillator, osc_index: usize) -> f32 {
    if note.to_remove {
        return 0.0;
    }

    let state = &mut note.osc_states[osc_index];
    if state.finished {
        return 0.0;
    }

    let freq: f32 = midi_to_freq(note.value) * osc.frequency_shift;

    let mut value = (state.current_phase * TAU).sin() * note.velocity as f32 / 127.0;

    apply_ADSR(osc, state, note.velocity, note.has_ended, &mut value);

    // avance la phase / compteurs uniquement pour CE state
    state.current_phase += freq / SAMPLE_RATE;
    state.current_phase %= 1.0;
    state.start_sample_index += 1;

    if note.has_ended {
        state.end_sample_index += 1;
    }

    value
}

fn apply_ADSR(
    osc: &Oscillator,
    state: &mut NoteOscState,
    note_velocity: u8,
    note_has_ended: bool,
    value: &mut f32,
) {
    if note_has_ended {
        if state.end_sample_index >= osc.release_length {
            state.finished = true; // cet osc est terminé
            *value = 0.0;
            return;
        }

        *value *= ((osc.release_length as f32 - state.end_sample_index as f32)
            / osc.release_length as f32);
    }

    if state.start_sample_index <= osc.attack_length {
        *value *= state.start_sample_index as f32 / osc.attack_length as f32;
    } else if state.start_sample_index <= osc.attack_length + osc.decay_length {
        *value *= 1.0
            + ((state.start_sample_index as f32 - osc.attack_length as f32)
                * (osc.sustain_gain - 1.0)
                / osc.decay_length as f32);
    } else {
        *value *= osc.sustain_gain;
    }
}

fn generate_samples(space: i32, current_sample_index_ref: &mut u64) -> Vec<f32> {
    let mut local_samples = Vec::with_capacity(space as usize);

    OSCILLATORS.with(|osc| {
        PLAYED_NOTES.with(|notes_cell| {
            let mut notes_mut = notes_cell.borrow_mut();
            let oscillators = osc.borrow();

            // suppression des notes mortes (tous les osc terminés)
            notes_mut.retain(|note| {
                let all_finished = note.osc_states.iter().all(|s| s.finished);
                if all_finished {
                    console::log_1(&format!("note {} supprimée", note.value).into());
                }
                !all_finished
            });

            for _ in 0..space {
                *current_sample_index_ref += 1;

                if notes_mut.is_empty() {
                    local_samples.push(0.0);
                    continue;
                }

                let mut osc_sum = 0.0;

                for note in notes_mut.iter_mut() {
                    let mut note_sum = 0.0;

                    for (osc_index, oscillator) in oscillators.iter().enumerate() {
                        note_sum += generate_sample_for_note(note, oscillator, osc_index);
                    }

                    osc_sum += note_sum; // on ajoute la contribution de cette note au mix final
                }

                local_samples.push(osc_sum * 0.1);
            }
        });
    });
    local_samples
}

fn write_to_ring_buffer(
    ring_buffer: &Float32Array,
    write_idx_atomic: &Int32Array,
    n: i32,
    local_samples: &[f32],
) {
    let space = local_samples.len() as i32;
    let mut current_write_idx = Atomics::load(write_idx_atomic, 0).unwrap();
    let chunk_array = Float32Array::from(local_samples);

    let contiguous_space = std::cmp::min(space, n - current_write_idx);

    if contiguous_space > 0 {
        ring_buffer.set(
            &chunk_array.subarray(0, contiguous_space as u32),
            current_write_idx as u32,
        );
    }
    if space > contiguous_space {
        let rest = space - contiguous_space;
        ring_buffer.set(
            &chunk_array.subarray(contiguous_space as u32, (contiguous_space + rest) as u32),
            0,
        );
        current_write_idx = rest;
    } else {
        current_write_idx = (current_write_idx + contiguous_space) % n;
    }

    Atomics::store(write_idx_atomic, 0, current_write_idx).unwrap();
}

fn fill_audio_chunk(
    space: i32,
    ring_buffer: &Float32Array,
    write_idx_atomic: &Int32Array,
    n: i32,
    current_sample_index_ref: &mut u64,
) {
    let local_samples = generate_samples(space, current_sample_index_ref);
    write_to_ring_buffer(ring_buffer, write_idx_atomic, n, &local_samples);
}

fn audio_producer_loop(buffers: &SharedBuffers) {
    let flag = &buffers.audio.flag;
    let read_idx = &buffers.audio.read_idx;
    let write_idx = &buffers.audio.write_idx;
    let ring_buffer = &buffers.audio.ring_buffer;
    let midi = &buffers.midi;

    let ring_buffer_len = ring_buffer.length() as i32;
    let mut global_sample_index: u64 = 0;

    console::log_1(&"Démarrage de la boucle audio (infinie)".into());

    loop {
        Atomics::wait(flag, 0, 1).unwrap();

        process_all_midi_events(midi, global_sample_index);

        let r_idx = Atomics::load(read_idx, 0).unwrap();
        let w_idx = Atomics::load(write_idx, 0).unwrap();

        let space_available = (r_idx - w_idx - 1 + ring_buffer_len) % ring_buffer_len;

        if space_available > 0 {
            fill_audio_chunk(
                space_available,
                ring_buffer,
                write_idx,
                ring_buffer_len,
                &mut global_sample_index,
            );
        }

        Atomics::store(flag, 0, 0).unwrap();
        Atomics::notify(flag, 0).unwrap();
    }
}

#[wasm_bindgen]
pub fn start_audio_processing_loop() {
    SHARED_BUFFERS.with(|cell| {
        let buffers = cell.get().expect("SharedBuffers not initialized!");
        audio_producer_loop(buffers);
    });
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
