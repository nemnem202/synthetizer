use crate::{
    global::MIXER,
    sound_engine::synthetizer::{note::Note, oscillator::Oscillator},
    utils::types::NoteDTO,
};

pub struct NoteManager {
    notes: Vec<Note>,
}

impl NoteManager {
    pub fn new() -> Self {
        Self { notes: Vec::new() }
    }

    pub fn add_note(&mut self, dto: &NoteDTO, oscillators: &[Oscillator]) {
        if let Some(existing_note) = self.notes.iter_mut().find(|n| n.value == dto.value) {
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
        self.notes.retain(|note| {
            let finished = note.is_finished();

            !finished
        });
    }

    pub fn generate_raw_samples(
        &mut self,
        sample_count: i32,
        oscillators: &[Oscillator],
    ) -> Vec<f32> {
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
                    let (l, r) = note.generate_samples_of_all_oscillators(oscillators);
                    mixed_l += l;
                    mixed_r += r;
                }
            }

            samples.push(mixed_l);
            samples.push(mixed_r);

            continue;
        }

        self.cleanup_finished_notes();
        samples
    }
}
