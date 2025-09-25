use once_cell::sync::Lazy;
use once_cell::sync::OnceCell;
use std::cell::RefCell;
use std::sync::Mutex;

use crate::shared_memory::shared_buffers::SharedBuffers;
use crate::sound_engine::dsp::mixer::Mixer;
use crate::sound_engine::processor::AudioProcessor;

thread_local! {
    pub static SHARED_BUFFERS: OnceCell<SharedBuffers> = OnceCell::new();
    pub static AUDIO_PROCESSOR: RefCell<Option<AudioProcessor>> = RefCell::new(None);

    pub static MIXER: Lazy<Mutex<Mixer>> = Lazy::new(|| {
        Mutex::new(Mixer::new())
    });
}
