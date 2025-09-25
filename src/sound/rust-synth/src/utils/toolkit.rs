use crate::utils::constants::{FREQ_A4, SAMPLE_RATE};

pub struct ToolKit;

impl ToolKit {
    pub fn midi_to_freq(note: u8) -> f32 {
        FREQ_A4 * 2.0f32.powf((note as f32 - 69.0) / 12.0)
    }

    pub fn convert_ms_to_sample(ms: f32) -> usize {
        (ms / 1000.0 * SAMPLE_RATE).floor() as usize
    }
}
