use wasm_bindgen::prelude::*;

use crate::{
    global::SAMPLE_MANAGER,
    sound_engine::synthetizer::note::NoteOscState,
    utils::{constants::SAMPLE_RATE, toolkit::ToolKit},
};

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct Sampler {
    pub id: u8,
    pub sample_id: u32,
    pub attack_length: u64,
    pub decay_length: u64,
    pub sustain_gain: f32,
    pub release_length: u64,
    pub frequency_shift: f32,
    pub phase_shift: f32,
    pub delay_length: u64,
    pub gain: f32,
    pub gain_l: f32,
    pub gain_r: f32,
}

impl Sampler {
    pub fn apply_adsr(&self, state: &mut NoteOscState, note_has_ended: bool, value: &mut f32) {
        if note_has_ended {
            if state.end_sample_index >= self.release_length + self.delay_length {
                state.finished = true;
                *value = 0.0;
                return;
            }

            *value *= (self.release_length as f32 - state.end_sample_index as f32)
                / self.release_length as f32;
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

        let freq: f32 = ToolKit::midi_to_freq(note_value) * self.frequency_shift;

        let mut value = SAMPLE_MANAGER.with(|sm| {
            sm.lock()
                .unwrap()
                .get_value(self.sample_id, state.start_sample_index, freq)
        }) * note_velocity as f32
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

        let left = value * self.gain_l;
        let right = value * self.gain_r;

        (left, right)
    }

    pub fn change_sample(&mut self, sample_id: u32) {
        self.sample_id = sample_id
    }
}
