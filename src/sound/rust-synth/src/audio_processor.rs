use js_sys::Atomics;
use web_sys::console;

use crate::buffers::MidiBuffers;
use crate::buffers::OscillatorBuffers;
use crate::constants::*;
use crate::note_manager::NoteManager;
use crate::ring_buffer_manager::RingBufferManager;
use crate::toolkit::*;

use crate::types::*;

use crate::oscillator::*;

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
                        attack_length: ToolKit::convert_ms_to_sample(500.0) as u64,
                        decay_length: ToolKit::convert_ms_to_sample(500.0) as u64,
                        sustain_gain: 0.5,
                        release_length: ToolKit::convert_ms_to_sample(500.0) as u64,
                        frequency_shift: 1.0,
                        delay_length: ToolKit::convert_ms_to_sample(0.0) as u64,
                        phase_shift: 0.0,
                        gain: 0.5,
                        gain_l: 1.0,
                        gain_r: 1.0,
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
                                osc.gain_l = (1.0 - value) / 2.0;
                                osc.gain_r = (1.0 + value) / 2.0
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
}
