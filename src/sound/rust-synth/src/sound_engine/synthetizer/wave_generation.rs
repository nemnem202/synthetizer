use std::f32::consts::TAU;

use crate::utils::types::WaveType;

pub struct WaveGenerator;

impl WaveGenerator {
    pub fn generate_sample(phase: f32, wave_type: WaveType) -> f32 {
        match wave_type {
            WaveType::Sine => (phase * TAU).sin(),
            WaveType::Square => {
                if phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            WaveType::Saw => 2.0 * (phase - 0.5),
            WaveType::Triangle => {
                let v = if phase < 0.5 {
                    4.0 * phase - 1.0
                } else {
                    3.0 - 4.0 * phase
                };
                v
            }
        }
    }
}
