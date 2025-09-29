use std::{cell::RefCell, rc::Rc};

use js_sys::{Atomics, Float32Array, Int32Array};
use web_sys::console;

use crate::{
    global::{MIXER, SAMPLE_MANAGER, SHARED_BUFFERS},
    shared_memory::shared_buffers::{FxBuffers, MidiBuffers, SamplerBuffers},
    sound_engine::{
        dsp::fx::EffectsEnum,
        synthetizer::{
            note_manager::{self, NoteManager},
            sampler::Sampler,
        },
    },
    utils::{
        constants::{
            FX_EVENT_SIZE_FLOAT, FX_EVENT_SIZE_INT, FX_QUEUE_CAPACITY, OSC_QUEUE_CAPACITY,
        },
        toolkit::ToolKit,
        types::SampleEvent,
    },
};

pub struct EventHandler {
    note_manager: Rc<RefCell<NoteManager>>,
    samplers: Rc<RefCell<Vec<Sampler>>>,
    last_sample_event: SampleEvent,
}

impl EventHandler {
    pub fn new(
        note_manager: Rc<RefCell<NoteManager>>,
        samplers: Rc<RefCell<Vec<Sampler>>>,
    ) -> Self {
        Self {
            note_manager,
            samplers,
            last_sample_event: SampleEvent::default(),
        }
    }

    pub fn process_midi_events(&mut self, midi: &MidiBuffers) -> u32 {
        midi.process_all_events(|dto| {
            if dto.velocity > 0 {
                self.note_manager
                    .borrow_mut()
                    .add_note(dto, &self.samplers.borrow_mut());
            } else {
                self.note_manager.borrow_mut().end_note(dto);
            }
        })
    }

    pub fn process_osc_events(&mut self, osc_buffers: &SamplerBuffers) {
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
                    self.samplers.borrow_mut().push(Sampler {
                        id: osc_index,
                        sample_id: 0,
                        attack_length: ToolKit::convert_ms_to_sample(0.0) as u64,
                        decay_length: ToolKit::convert_ms_to_sample(10.0) as u64,
                        sustain_gain: 0.5,
                        release_length: ToolKit::convert_ms_to_sample(500.0) as u64,
                        frequency_shift: 1.0,
                        delay_length: ToolKit::convert_ms_to_sample(0.0) as u64,
                        phase_shift: 0.0,
                        gain: 0.5,
                        gain_l: 1.0,
                        gain_r: 1.0,
                    });
                    console::log_1(&format!("Oscillateur créé à l'index {}", osc_index).into());
                }
                1 => {
                    // remove
                    if let Some(pos) = self
                        .samplers
                        .borrow_mut()
                        .iter()
                        .position(|osc| osc.id == osc_index)
                    {
                        self.samplers.borrow_mut().remove(pos);
                        console::log_1(&format!("Oscillateur {} supprimé", osc_index).into());
                    } else {
                        console::log_1(&format!("Oscillateur {} introuvable", osc_index).into());
                    }
                }
                2 => {
                    // update
                    if let Some(osc) = self
                        .samplers
                        .borrow_mut()
                        .iter_mut()
                        .find(|o| o.id == osc_index)
                    {
                        match key {
                            1 => osc.attack_length = value as u64,
                            2 => osc.release_length = value as u64,
                            3 => osc.decay_length = value as u64,
                            4 => osc.sustain_gain = value * 0.1,
                            5 => osc.gain = value * 0.1,
                            6 => osc.delay_length = value as u64,
                            7 => osc.frequency_shift = value,
                            8 => osc.phase_shift = value,
                            9 => osc.sample_id = value as u32,
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

    pub fn process_fx_events(&mut self, fx_buffer: &FxBuffers) {
        let mut read_pos = Atomics::load(&fx_buffer.read_idx, 0).unwrap() as u32;
        let write_pos = Atomics::load(&fx_buffer.write_idx, 0).unwrap() as u32;

        if read_pos == write_pos {
            return;
        }

        while read_pos != write_pos {
            // Offsets pour les vues séparées
            let int_offset = read_pos * FX_EVENT_SIZE_INT;
            let float_offset = read_pos * FX_EVENT_SIZE_FLOAT;

            let fx_id = fx_buffer.queue_int.get_index(int_offset) as u32;
            let event_type = fx_buffer.queue_int.get_index(int_offset + 1) as u32;
            let param_index = fx_buffer.queue_int.get_index(int_offset + 2) as u32;

            let value = fx_buffer.queue_float.get_index(float_offset);

            match event_type {
                0 => self.add_fx(fx_id, param_index),
                1 => self.remove_fx(fx_id),
                2 => self.edit_fx(fx_id, param_index, value),
                _ => {}
            }

            read_pos = (read_pos + 1) % FX_QUEUE_CAPACITY;
        }

        Atomics::store(&fx_buffer.read_idx, 0, read_pos as i32).unwrap();
    }

    pub fn add_fx(&mut self, fx_id: u32, param_index: u32) {
        let effect = EffectsEnum::try_from(param_index).unwrap();

        match effect {
            EffectsEnum::Echo => MIXER.with(|m| {
                let mut mixer = m.lock().unwrap();
                mixer.create_echo(fx_id);
            }),
            EffectsEnum::Filter => {
                console::log_1(&format!("Filtre créé à l'id {}", fx_id).into());
            }
        }
    }

    pub fn remove_fx(&mut self, fx_id: u32) {
        MIXER.with(|m| {
            let mut mixer = m.lock().unwrap();
            mixer.remove_fx(fx_id);
            console::log_1(&format!("Effet avec l'id {} retiré", fx_id).into());
        })
    }

    pub fn edit_fx(&mut self, fx_id: u32, param_index: u32, value: f32) {
        MIXER.with(|m| {
            let mut mixer = m.lock().unwrap();
            mixer.update_fx(fx_id, param_index, value);
        })
    }

    pub fn process_sample_event(&mut self, sample_event: &Int32Array) {
        let sample_event_index = sample_event.get_index(0);

        // si on change vraiment de sampler ou sample
        if sample_event_index as u32 != self.last_sample_event.sample_event_index {
            console::log_1(&"traitement du sample buffer".into());
            let new_event = SampleEvent {
                sample_event_index: sample_event_index as u32,
                sampler_id: sample_event.get_index(1) as u32,
                sample_id: sample_event.get_index(2) as u32,
                length: sample_event.get_index(3) as u32,
                channels: sample_event.get_index(4) as u8,
            };
            self.last_sample_event = new_event;

            SAMPLE_MANAGER.with(|sm| {
                let mut sm = sm.lock().unwrap();

                SHARED_BUFFERS.with(|sb| {
                    if let Some(shared) = sb.get() {
                        let sample_buffer: &Float32Array = &shared.sample_buffer;

                        // Ici tu peux ajouter ton sample
                        sm.add_sample(
                            self.last_sample_event.sample_id,
                            sample_buffer.clone(),
                            self.last_sample_event.length,
                        );
                    }
                });
            });

            // et ici on peut utiliser self.last_sample_event
            if let Some(sampler) = self
                .samplers
                .borrow_mut()
                .iter_mut()
                .find(|s| s.id == self.last_sample_event.sampler_id as u8)
            {
                sampler.sample_id = self.last_sample_event.sample_id;
            }
        }
    }
}
