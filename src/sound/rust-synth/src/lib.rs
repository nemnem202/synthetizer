use std::cell::RefCell;
use std::sync::Mutex;

use js_sys::{Atomics, Float32Array, Int32Array, SharedArrayBuffer, Uint8Array};
use once_cell::sync::Lazy;
use once_cell::sync::OnceCell;
use wasm_bindgen::prelude::*;
use web_sys::console;

pub mod buffers;
use crate::buffers::*;
pub mod audio_processor;
pub mod constants;
pub mod dsp_effects;
pub mod mixer;
pub mod note;
pub mod note_manager;
pub mod oscillator;
pub mod ring_buffer_manager;
pub mod toolkit;
pub mod types;
pub mod wave_gen;

use crate::constants::*;
use crate::mixer::Mixer;
use crate::ring_buffer_manager::*;

use crate::audio_processor::*;

use console_error_panic_hook;

thread_local! {
    pub static SHARED_BUFFERS: OnceCell<SharedBuffers> = OnceCell::new();
    pub static AUDIO_PROCESSOR: RefCell<Option<AudioProcessor>> = RefCell::new(None);

pub static MIXER: Lazy<Mutex<Mixer>> = Lazy::new(|| {
    Mutex::new(Mixer::new())
});
}

#[wasm_bindgen(start)]
pub fn main() {
    // Installe le hook pour que les panics soient affichés dans la console JS
    console_error_panic_hook::set_once();

    // Lancement du reste de ton code
}

#[wasm_bindgen]
pub fn init_audio_thread(
    shared_audio_buffer: SharedArrayBuffer,
    ring_buffer_size: u32,
    midi_buffer: SharedArrayBuffer,
    osc_buffer: SharedArrayBuffer,
    fx_buffer: SharedArrayBuffer,
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

    // -------- FX ------------------

    let fx_control_arr = Int32Array::new(&fx_buffer);
    let fx_write_idx = fx_control_arr.subarray(0, 1);
    let fx_read_idx = fx_control_arr.subarray(1, 2);

    // 2 Int32 pour write_idx + read_idx
    let fx_int_offset = 2 * 4; // 2 Int32 * 4 octets
    let fx_float_offset = fx_int_offset + 3 * FX_QUEUE_CAPACITY * 4; // 3 Int32 par event * 64 events * 4 octets

    let fx_queue_int_full = Int32Array::new(&fx_buffer);
    let fx_queue_float_full = Float32Array::new(&fx_buffer);

    let fx_queue_int = fx_queue_int_full.subarray(
        fx_int_offset / 4, // offset en nombre d'éléments
        fx_int_offset / 4 + 3 * FX_QUEUE_CAPACITY,
    );

    let fx_queue_float = fx_queue_float_full.subarray(
        fx_float_offset / 4, // offset en nombre d'éléments
        fx_float_offset / 4 + FX_QUEUE_CAPACITY,
    );

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
        fx: FxBuffers {
            write_idx: fx_write_idx,
            read_idx: fx_read_idx,
            queue_int: fx_queue_int,
            queue_float: fx_queue_float,
        },
    };

    _ = SHARED_BUFFERS.with(|cell| cell.set(shared_buffers));
    // Initialisation du processeur audio avec les oscillateurs de test
    let audio_processor = AudioProcessor::new();

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
                processor.process_midi_events(midi);

                processor.process_osc_events(&buffers.osc);

                processor.process_fx_events(&buffers.fx);
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
