use std::f32::consts::PI;
use wasm_bindgen::prelude::*;

const WINDOW_SIZE: usize = 128;
const ORIGINAL_NOTE: i32 = 60;
const TARGET_NOTE: i32 = 12;

/// Fonction sinc(x)
fn sinc(x: f32) -> f32 {
    if x == 0.0 {
        1.0
    } else {
        (PI * x).sin() / (PI * x)
    }
}

/// Fenêtre de Hamming
fn hamming_window(n: usize, N: usize) -> f32 {
    let nf = n as f32;
    let Nf = N as f32;
    0.54 - 0.46 * ((2.0 * PI * nf) / (Nf - 1.0)).cos()
}

/// Interpolation sinc
fn sinc_interpolation(samples: &[f32], target_index: f32) -> f32 {
    let mut result = 0.0;
    let mut denominator = 0.0;
    let half_window = (WINDOW_SIZE / 2) as isize;

    let target_index_floor = target_index.floor() as isize;

    for i in -half_window..=half_window {
        let sample_index = target_index_floor + i;
        if sample_index >= 0 && (sample_index as usize) < samples.len() {
            let x = target_index - sample_index as f32;
            let sinc_value = sinc(x);
            let window_value = hamming_window((i + half_window) as usize, WINDOW_SIZE);
            result += samples[sample_index as usize] * sinc_value * window_value;
            denominator += sinc_value * window_value;
        }
    }

    if denominator != 0.0 {
        result / denominator
    } else {
        0.0
    }
}

#[wasm_bindgen]
pub fn generate_c0_table(samples: &[f32]) -> Vec<f32> {
    let semitone_diff = (ORIGINAL_NOTE - TARGET_NOTE) as f32;
    let factor = 2f32.powf(semitone_diff / 12.0);
    let new_length = (samples.len() as f32 * factor).floor() as usize;

    let mut result = Vec::with_capacity(new_length);
    for i in 0..new_length {
        let original_index = i as f32 / factor;
        let value = sinc_interpolation(samples, original_index);
        result.push(value);
    }

    result
}
