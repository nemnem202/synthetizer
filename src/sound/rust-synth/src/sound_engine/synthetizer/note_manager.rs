use web_sys::console;

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
        output_buffer: &mut [f32],
        frame_count: usize, // C'est le nombre de frames stéréo
        oscillators: &[Oscillator],
    ) {
        output_buffer.fill(0.0);

        if output_buffer.len() < frame_count * 2 {
            console::error_1(
                &format!(
                    "Output buffer in generate_raw_samples is too small for {} frames!",
                    frame_count
                )
                .into(),
            );
            return;
        }

        for i in 0..frame_count {
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

            output_buffer[i * 2] = mixed_l;
            output_buffer[i * 2 + 1] = mixed_r;

            continue;
        }

        self.cleanup_finished_notes();
    }
}
