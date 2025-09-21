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
}

static TEST_OSCILLATOR: Oscillator = Oscillator {
    wave_type: WaveType::Sine,
    attack_length: 100,
    decay_length: 3000,
    sustain_gain: 0.0,
    release_length: 10000,
};

static TEST_OSCILLATOR_2: Oscillator = Oscillator {
    wave_type: WaveType::Sine,
    attack_length: 200,
    decay_length: 5000,
    sustain_gain: 0.5,
    release_length: 100,
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
    pub static OSCILLATORS: RefCell<Vec<Oscillator>> = RefCell::new(vec![TEST_OSCILLATOR]);
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct NoteDTO {
    pub value: u8,
    pub velocity: u8,
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
}

impl Note {
    fn new(value: u8, velocity: u8, start_sample_index: u64) -> Self {
        Note {
            value,
            velocity,
            has_ended: false,
            to_remove: false,
            start_sample_index: 0,
            end_sample_index: 0,
            current_phase: 0.0,
        }
    }
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
            }
        } else {
            console::log_1(&format!("On push la note dans le tableau").into());
            notes_mut.push(Note::new(dto.value, dto.velocity, current_sample_index));
        }
    });
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

fn generate_sample_for_note(note: &mut Note, osc: &Oscillator) -> f32 {
    if note.to_remove == true {
        return 0.0;
    }
    let freq = midi_to_freq(note.value);

    let mut value = (note.current_phase * TAU).sin() * note.velocity as f32 / 127.0;

    apply_ADSR(osc, note, &mut value);

    note.current_phase += freq / SAMPLE_RATE;
    note.start_sample_index += 1;
    note.current_phase %= 1.0;
    if (note.has_ended) {
        note.end_sample_index += 1
    }
    value
}

fn apply_ADSR(osc: &Oscillator, note: &mut Note, value: &mut f32) {
    if note.has_ended {
        if (osc.release_length == note.end_sample_index) {
            note.to_remove = true;
            *value = 0.0;
            return;
        }

        *value *= ((osc.release_length as f32 - note.end_sample_index as f32)
            / osc.release_length as f32);
    }

    if (note.start_sample_index <= osc.attack_length) {
        *value *= note.start_sample_index as f32 / osc.attack_length as f32
    } else if (note.start_sample_index <= osc.attack_length + osc.decay_length) {
        *value *= 1.0
            + ((note.start_sample_index as f32 - osc.attack_length as f32)
                * (osc.sustain_gain - 1.0)
                / osc.decay_length as f32)
    } else if (note.start_sample_index >= osc.attack_length + osc.decay_length) {
        *value *= osc.sustain_gain
    }
}

fn generate_samples(space: i32, current_sample_index_ref: &mut u64) -> Vec<f32> {
    let mut local_samples = Vec::with_capacity(space as usize);

    OSCILLATORS.with(|osc| {
        PLAYED_NOTES.with(|notes_cell| {
            let mut notes_mut = notes_cell.borrow_mut();

            notes_mut.retain(|note| {
                let keep = !note.to_remove;
                if (note.to_remove) {
                    console::log_1(&format!("note supprimee").into());
                }
                keep
            });

            let oscillators = osc.borrow();

            for _ in 0..space {
                *current_sample_index_ref += 1;

                if notes_mut.is_empty() {
                    local_samples.push(0.0);
                    continue;
                }

                let mut osc_sum = 0.0;
                for oscillator in oscillators.iter() {
                    let note_sum: f32 = notes_mut
                        .iter_mut()
                        .map(|note| generate_sample_for_note(note, oscillator))
                        .sum();
                    osc_sum += note_sum;
                }

                // let norm_factor = (notes_mut.len() * oscillators.len()).max(1) as f32;
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
