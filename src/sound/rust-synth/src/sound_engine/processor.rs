use std::{cell::RefCell, rc::Rc};

use crate::{
    global::MIXER,
    shared_memory::ring_buffer_manager::RingBufferManager,
    sound_engine::{
        event_handler::{self, EventHandler},
        synthetizer::{
            note_manager::{self, NoteManager},
            oscillator::{self, Oscillator},
        },
    },
};

pub struct AudioProcessor {
    pub note_manager: Rc<RefCell<NoteManager>>,
    pub oscillators: Rc<RefCell<Vec<Oscillator>>>,
    pub event_handler: EventHandler,
    pub global_sample_index: u64,
}

impl AudioProcessor {
    pub fn new() -> Self {
        let note_manager = Rc::new(RefCell::new(NoteManager::new()));
        let oscillators = Rc::new(RefCell::new(Vec::new()));
        let event_handler = EventHandler::new(Rc::clone(&note_manager), Rc::clone(&oscillators));

        Self {
            note_manager,
            oscillators,
            event_handler,
            global_sample_index: 0,
        }
    }

    pub fn process_and_fill_audio_buffer(
        &mut self,
        sample_count: i32,
        ring_buffer_manager: &RingBufferManager,
    ) {
        self.global_sample_index += sample_count as u64;
        let mut samples = self
            .note_manager
            .borrow_mut()
            .generate_raw_samples(sample_count, &self.oscillators.borrow());

        self.apply_final_mixing(&mut samples);

        ring_buffer_manager.write_samples(&samples);
    }

    pub fn apply_final_mixing(&self, raw_samples: &mut Vec<f32>) {
        // Option 1: Boucle for classique (recommandée pour l'indexation par pas de 2)
        for i in (0..raw_samples.len()).step_by(2) {
            let mut mixed_l = raw_samples[i];
            let mut mixed_r = raw_samples[i + 1];

            // Normalisation par nombre d'oscillateurs
            if !self.oscillators.borrow().is_empty() {
                let osc_count = self.oscillators.borrow().len() as f32;
                mixed_l /= osc_count;
                mixed_r /= osc_count;
            }

            MIXER.with(|mix| {
                mix.lock().unwrap().render(&mut mixed_l, &mut mixed_r);
            });

            mixed_l *= 0.1;
            mixed_r *= 0.1;

            // Réécrire dans le Vec
            raw_samples[i] = mixed_l;
            raw_samples[i + 1] = mixed_r;
        }
    }
}
