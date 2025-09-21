use js_sys::{Atomics, Float32Array, Int8Array, Int32Array, SharedArrayBuffer, Uint8Array};
use once_cell::unsync::OnceCell;
use std::cell::RefCell;
use std::f32::consts::TAU;
use wasm_bindgen::prelude::*;
use web_sys::console;

thread_local! {
    static FLAG: OnceCell<Int32Array> = OnceCell::new();
    static READ_INDEX: OnceCell<Int32Array> = OnceCell::new();
    static WRITE_INDEX: OnceCell<Int32Array> = OnceCell::new();
    static RING_BUFFER: OnceCell<Float32Array> = OnceCell::new();

    static NOTES_EVENTS: OnceCell<Uint8Array> = OnceCell::new();
    pub static PLAYED_NOTES: RefCell<Vec<Note>> = RefCell::new(Vec::new());
}

#[wasm_bindgen]
pub fn init_buffer(
    shared: SharedArrayBuffer,
    ring_buffer_size: u32,
    notes_shared: SharedArrayBuffer,
) {
    // définir le tableau de notes
    let notes_events = Uint8Array::new(&notes_shared);
    NOTES_EVENTS.with(|c| {
        let _ = c.set(notes_events);
    });

    // first 3 Int32 : flag, readIndex, writeIndex
    let indexes = Int32Array::new(&shared).subarray(0, 3);

    let flag = indexes.subarray(0, 1);
    let read_index = indexes.subarray(1, 2);
    let write_index = indexes.subarray(2, 3);

    // offset en éléments Float32 après les 3 Int32 (3 * 4 bytes) => /4 pour index en éléments
    let indexes_bytes = 3 * 4; // 3 * sizeof(Int32)
    let start_elem = (indexes_bytes / 4) as u32;
    let ring_end = start_elem + ring_buffer_size;

    let ring = Float32Array::new(&shared).subarray(start_elem, ring_end);

    FLAG.with(|c| {
        let _ = c.set(flag);
    });
    READ_INDEX.with(|c| {
        let _ = c.set(read_index);
    });
    WRITE_INDEX.with(|c| {
        let _ = c.set(write_index);
    });
    RING_BUFFER.with(|c| {
        let _ = c.set(ring);
    });
}

#[wasm_bindgen]
pub fn init_loop() {
    get_flag();
}

#[wasm_bindgen]
pub struct NoteDTO {
    pub value: u8,    // 0..127
    pub velocity: u8, // 0..127
}

pub enum WaveType {
    Sine,
    Square,
    Saw,
    Triangle,
}

pub struct OscillatorDTO {
    pub wave: WaveType,
}
// La note qui est réellement stockée dans PlayedNotes
#[derive(Clone)]
pub struct Note {
    pub value: u8,
    pub velocity: u8,
    pub has_ended: bool,
    pub index_sample_from_start: u64,
    pub index_sample_it_ended: u64,
}

pub fn add_note(dto: &NoteDTO) {
    let msg = format!(
        "nouvelle note, valeur de la note: {}, fréquence de la note: {}",
        dto.value,
        midi_to_freq(dto.value)
    );
    console::log_1(&JsValue::from_str(&msg));
    PLAYED_NOTES.with(|notes| {
        let mut notes = notes.borrow_mut(); // on emprunte en mode "mutable"
        notes.push(Note {
            value: dto.value,
            velocity: dto.velocity,
            has_ended: false,
            index_sample_from_start: 0,
            index_sample_it_ended: 0,
        });
    });
}

pub fn set_note_end(dto: &NoteDTO) {
    PLAYED_NOTES.with(|notes| {
        let mut notes = notes.borrow_mut();
        for note in notes.iter_mut() {
            if note.value == dto.value {
                note.has_ended = true; // ou 0 si tu veux un int
            }
        }
    });
}

fn increment_note_index_from_start() {
    PLAYED_NOTES.with(|notes| {
        let mut notes = notes.borrow_mut();
        for note in notes.iter_mut() {
            note.index_sample_from_start += 1;
        }
    });
}

fn get_flag() {
    FLAG.with(|flag_cell| {
        let flag = flag_cell.get().expect("flag not init !");
        get_read_index(flag);
    })
}

fn get_read_index(flag: &Int32Array) {
    READ_INDEX.with(|read_cell| {
        let read_index = read_cell.get().expect("read index not init !");
        get_write_index(flag, read_index);
    })
}

fn get_write_index(flag: &Int32Array, read_index: &Int32Array) {
    WRITE_INDEX.with(|write_cell| {
        let write_index = write_cell.get().expect("write index not init !");
        get_ring_buffer(flag, read_index, write_index);
    })
}

fn get_ring_buffer(flag: &Int32Array, read_index: &Int32Array, write_index: &Int32Array) {
    RING_BUFFER.with(|ring_cell| {
        let ring_buffer = ring_cell.get().expect("ring buffer not init !");
        fill_buffer_when_flag_changes(flag, read_index, write_index, ring_buffer);
    })
}

fn fill_buffer_when_flag_changes(
    flag: &Int32Array,
    read_index: &Int32Array,
    write_index: &Int32Array,
    ring_buffer: &Float32Array,
) {
    let n = ring_buffer.length() as i32;

    let freq: f32 = 440.0;
    let sample_rate: f32 = 44100.0;
    let mut phase: f32 = 0.0;
    // boucle de production : bloque le thread via Atomics.wait
    loop {
        let _ = Atomics::wait(flag, 0, 1);

        let current_flag = Atomics::load(flag, 0).unwrap();

        if current_flag == 2 {
            NOTES_EVENTS.with(|cell| {
                if let Some(arr) = cell.get() {
                    let event_type = arr.get_index(0);
                    let note = arr.get_index(1);
                    let vel = arr.get_index(2);

                    let dto = NoteDTO {
                        value: note,   // exemple : E4
                        velocity: vel, // exemple : forte
                    };

                    if (event_type == 1) {
                        add_note(&dto);
                    } else if (event_type == 0) {
                        set_note_end(&dto);
                    }
                } else {
                    console::log_1(&"NOTE_EVENTS non initialisé".into());
                }
            });

            // remet le flag à 1
            let _ = Atomics::store(flag, 0, 1);
            let _ = Atomics::notify(flag, 0);
            continue;
        }

        let r_js = Atomics::load(read_index, 0).unwrap();
        let w_js = Atomics::load(write_index, 0).unwrap();

        let r_index = r_js;
        let mut w_index = w_js;

        let space = ((r_index - w_index - 1 + n) % n) as i32;

        if space > 0 {
            fill_space(
                space,
                n,
                phase,
                freq,
                sample_rate,
                write_index,
                w_index,
                ring_buffer,
            );
        }

        let _ = Atomics::store(flag, 0, 1);
        let _ = Atomics::notify(flag, 0);
    }
}

pub fn fill_space(
    space: i32,
    n: i32,
    mut phase: f32,
    freq: f32,
    sample_rate: f32,
    write_index: &Int32Array,
    mut w_index: i32,
    ring_buffer: &Float32Array,
) {
    let mut local: Vec<f32> = Vec::with_capacity(space as usize);

    for i in 0..space {
        fill_one_sample(phase, &mut local, freq, sample_rate);
        increment_note_index_from_start();
    }

    let new_w_index = fill_ring_buffer(space, w_index, ring_buffer, &local, n);

    let _ = Atomics::store(write_index, 0, new_w_index);
}

fn fill_ring_buffer(
    space: i32,
    mut w_index: i32,
    ring_buffer: &Float32Array,
    local: &Vec<f32>,
    n: i32,
) -> i32 {
    let chunk_array = Float32Array::from(local.as_slice());
    let contiguous = std::cmp::min(space, n - w_index);
    if contiguous > 0 {
        ring_buffer.set(&chunk_array.subarray(0, contiguous as u32), w_index as u32);
    }
    if space as i32 > contiguous {
        let rest = space - contiguous;
        ring_buffer.set(
            &chunk_array.subarray(contiguous as u32, (contiguous + rest) as u32),
            0,
        );
        w_index = rest as i32;
    } else {
        w_index = (w_index + contiguous) % n;
    }
    w_index
}

fn fill_one_sample(mut phase: f32, local: &mut Vec<f32>, freq: f32, sample_rate: f32) {
    PLAYED_NOTES.with(|notes| {
        let mut notes = notes.borrow_mut();

        if notes.is_empty() {
            local.push(0.0);
            return;
        }

        let mut sample: f32 = 0.0;

        for note in notes.iter_mut() {
            if (note.has_ended == true && note.index_sample_it_ended == 0) {
                note.index_sample_it_ended = note.index_sample_from_start
            }

            let mut note_freq = midi_to_freq(note.value);

            let mut note_phase =
                phase_from_sample_index(note_freq, note.index_sample_from_start, sample_rate);

            let value = (note_phase * TAU).sin() * note.velocity as f32 / 127.0;

            sample += value
        }

        // sample /= notes.len() as f32;

        local.push(sample);
    });
}

fn midi_to_freq(note: u8) -> f32 {
    440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0)
}

fn phase_from_sample_index(freq: f32, index: u64, sample_rate: f32) -> f32 {
    (freq * index as f32 / sample_rate) % 1.0
}
