use crate::sound_engine::synthetizer::sampler::Sampler;

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
    pub fn new(value: u8, velocity: u8, samplers: &[Sampler]) -> Self {
        let osc_states = samplers
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

    pub fn restart(&mut self, samplers: &[Sampler]) {
        self.has_ended = false;
        self.end_sample_index = 0;
        self.start_sample_index = 0;

        // Réajuster si le nombre d'samplers a changé
        if self.osc_states.len() != samplers.len() {
            self.osc_states = samplers
                .iter()
                .map(|osc| NoteOscState::new(osc.phase_shift))
                .collect();
        } else {
            for (state, osc) in self.osc_states.iter_mut().zip(samplers.iter()) {
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

    pub fn generate_samples_of_all_samplers(&mut self, samplers: &[Sampler]) -> (f32, f32) {
        if self.to_remove {
            return (0.0, 0.0);
        }

        let mut note_sum_l = 0.0;
        let mut note_sum_r = 0.0;

        for (osc_index, sampler) in samplers.iter().enumerate() {
            if let Some(state) = self.osc_states.get_mut(osc_index) {
                let (l, r) =
                    sampler.generate_sample(self.value, self.velocity, state, self.has_ended);
                note_sum_l += l;
                note_sum_r += r;
            }
        }

        (note_sum_l, note_sum_r)
    }
}
