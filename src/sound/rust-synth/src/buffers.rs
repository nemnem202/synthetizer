use js_sys::{Atomics, Float32Array, Int32Array, Uint8Array};

use crate::constants::*;

use crate::types::*;

pub struct AudioBuffers {
    pub flag: Int32Array,
    pub read_idx: Int32Array,
    pub write_idx: Int32Array,
    pub ring_buffer: Float32Array,
}

pub struct MidiBuffers {
    pub write_idx: Int32Array,
    pub read_idx: Int32Array,
    pub queue: Uint8Array,
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

pub struct FxBuffers {
    pub write_idx: Int32Array,
    pub read_idx: Int32Array,
    pub queue_int: Int32Array,     // fx_id, param_index, event_type
    pub queue_float: Float32Array, // value
}

pub struct FxEventDto {
    pub id: u32,
    pub param_index: u32,
    pub event_type: u32,
    pub value: f32,
}

impl FxBuffers {
    pub fn dequeue_event(&self) -> Option<FxEventDto> {
        let read_pos = Atomics::load(&self.read_idx, 0).unwrap() as u32;
        let write_pos = Atomics::load(&self.write_idx, 0).unwrap() as u32;

        if read_pos == write_pos {
            return None;
        }

        let int_offset = read_pos * 3; // 3 int par événement
        let float_offset = read_pos; // 1 float par événement

        let fx_id = self.queue_int.get_index(int_offset) as u32;
        let event_type = self.queue_int.get_index(int_offset + 1) as u32;
        let param_index = self.queue_int.get_index(int_offset + 2) as u32;

        let value = self.queue_float.get_index(float_offset);

        let new_read_pos = (read_pos + 1) % FX_QUEUE_CAPACITY;
        Atomics::store(&self.read_idx, 0, new_read_pos as i32).unwrap();

        Some(FxEventDto {
            id: fx_id,
            param_index,
            event_type,
            value,
        })
    }

    pub fn process_all_events<F>(&self, mut handler: F) -> u32
    where
        F: FnMut(&FxEventDto),
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
    pub write_idx: Int32Array,
    pub read_idx: Int32Array,
    pub queue: Uint8Array, // 8 octets par événement
}

pub struct SharedBuffers {
    pub audio: AudioBuffers,
    pub midi: MidiBuffers,
    pub osc: OscillatorBuffers,
    pub fx: FxBuffers,
}

pub struct MemoryBuffer {
    pub buffer: Vec<f32>,
    pub size: usize,
    pub write_index: usize,
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

    pub fn write(&mut self, sample_l: f32, sample_r: f32) {
        self.buffer[self.write_index] = sample_l;
        self.buffer[self.write_index + 1] = sample_r;
        self.write_index = (self.write_index + 2) % self.size;
    }

    pub fn read_mono(&self, delay_samples: usize) -> (f32, f32) {
        let read_index = (self.size + self.write_index - delay_samples) % self.size;
        (self.buffer[read_index], self.buffer[read_index + 1])
    }

    pub fn read_left(&self, delay_samples: usize) -> f32 {
        // On recule de delay_samples * 2 cases (car stéréo)
        let read_index = (self.size + self.write_index - delay_samples * 2) % self.size;
        self.buffer[read_index]
    }

    pub fn read_right(&self, delay_samples: usize) -> f32 {
        let read_index = (self.size + self.write_index - delay_samples * 2) % self.size;
        // Toujours dans la paire stéréo
        self.buffer[(read_index + 1) % self.size]
    }

    // pub fn read_left(&self, delay_samples: usize) -> f32 {
    //     let read_index = (self.size + self.write_index - delay_samples) % self.size;
    //     self.buffer[read_index]
    // }

    // pub fn read_right(&self, delay_samples: usize) -> f32 {
    //     let read_index = (self.size + self.write_index - delay_samples) % self.size;
    //     self.buffer[read_index + 1]
    // }
}
