use std::{cell::RefCell, rc::Rc};

use web_sys::console;

use crate::{
    global::MIXER,
    shared_memory::ring_buffer_manager::RingBufferManager,
    sound_engine::{
        event_handler::{self, EventHandler},
        synthetizer::{
            note_manager::{self, NoteManager},
            sampler::{self, Sampler},
        },
    },
    utils::constants::PROCESSING_BUFFER_SIZE,
};

pub struct AudioProcessor {
    pub note_manager: Rc<RefCell<NoteManager>>,
    pub samplers: Rc<RefCell<Vec<Sampler>>>,
    pub event_handler: EventHandler,
    pub global_sample_index: u64,
    pub processing_buffer: Vec<f32>, // Alloué une seule fois
    pub processing_buffer_size: usize,
}

impl AudioProcessor {
    pub fn new() -> Self {
        let note_manager = Rc::new(RefCell::new(NoteManager::new()));
        let samplers = Rc::new(RefCell::new(Vec::new()));
        let event_handler = EventHandler::new(Rc::clone(&note_manager), Rc::clone(&samplers));

        Self {
            note_manager,
            samplers,
            event_handler,
            global_sample_index: 0,
            processing_buffer: vec![0.0; PROCESSING_BUFFER_SIZE * 2],
            processing_buffer_size: PROCESSING_BUFFER_SIZE,
        }
    }

    pub fn process_and_fill_audio_buffer(
        &mut self,
        frame_count: i32, // Renommé pour la clarté : c'est le nombre de frames stéréo
        ring_buffer_manager: &RingBufferManager,
    ) {
        let num_elements_f32 = frame_count * 2; // C'est le nombre d'éléments f32 à traiter
        if num_elements_f32 as usize > self.processing_buffer.len() {
            console::error_1(
                &format!(
                    "Processing buffer too small! Required: {}, Actual: {}",
                    num_elements_f32,
                    self.processing_buffer.len()
                )
                .into(),
            );
            return;
        }

        self.global_sample_index += frame_count as u64; // C'est le nombre de frames
        let samples_slice = &mut self.processing_buffer[0..num_elements_f32 as usize];

        self.note_manager.borrow_mut().generate_raw_samples(
            samples_slice,
            frame_count as usize, // Passe le nombre de frames à generate_raw_samples
            &self.samplers.borrow(),
        );

        AudioProcessor::apply_final_mixing(samples_slice, &self.samplers);

        ring_buffer_manager.write_samples(samples_slice);
    }

    pub fn apply_final_mixing(raw_samples: &mut [f32], samplers: &RefCell<Vec<Sampler>>) {
        // Option 1: Boucle for classique (recommandée pour l'indexation par pas de 2)
        for i in (0..raw_samples.len()).step_by(2) {
            let mut mixed_l = raw_samples[i];
            let mut mixed_r = raw_samples[i + 1];

            // Normalisation par nombre d'samplers
            if !samplers.borrow().is_empty() {
                let osc_count = samplers.borrow().len() as f32;
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
