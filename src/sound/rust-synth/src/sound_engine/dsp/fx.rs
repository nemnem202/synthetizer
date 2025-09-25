use std::any::Any;

use crate::utils::{constants::SAMPLE_RATE, types::Mix};

pub struct MemoryBuffer {
    pub buffer: Vec<f32>,
    pub size: usize,
    pub write_index: usize,
}

impl MemoryBuffer {
    /// Crée un buffer pour `duration_seconds` à `sample_rate` Hz
    pub fn new(sample_rate: usize, duration_seconds: f32) -> Self {
        let size = (sample_rate as f32 * duration_seconds * 2.0) as usize;
        Self {
            buffer: vec![0.0; size],
            size,
            write_index: 0,
        }
    }

    pub fn write(&mut self, sample_l: f32, sample_r: f32) {
        self.buffer[self.write_index] = sample_l;
        self.buffer[self.write_index + 1] = sample_r;
        self.write_index = (self.write_index + 2) % self.size;
    }

    pub fn read_mono(&self, delay_samples: usize) -> (f32, f32) {
        let read_index = (self.size + self.write_index - delay_samples) % self.size;
        (self.buffer[read_index], self.buffer[read_index + 1])
    }

    pub fn read_left(&self, delay_samples: usize) -> f32 {
        // On recule de delay_samples * 2 cases (car stéréo)
        let read_index = (self.size + self.write_index - delay_samples * 2) % self.size;
        self.buffer[read_index]
    }

    pub fn read_right(&self, delay_samples: usize) -> f32 {
        let read_index = (self.size + self.write_index - delay_samples * 2) % self.size;
        // Toujours dans la paire stéréo
        self.buffer[(read_index + 1) % self.size]
    }
}

pub enum EffectsEnum {
    Echo,
    Filter,
}

impl TryFrom<u32> for EffectsEnum {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EffectsEnum::Echo),
            1 => Ok(EffectsEnum::Filter),
            _ => Err(()),
        }
    }
}
pub trait EffectTrait: Any {
    fn id(&self) -> usize;
    fn process(&mut self, sample_l: &mut f32, sample_r: &mut f32);

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct BiquadCoeffs {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
}

impl BiquadCoeffs {
    pub fn calc_biquad_coeffs(frequency: f32, q: f32) -> BiquadCoeffs {
        let w0 = 2.0 * std::f32::consts::PI * frequency / SAMPLE_RATE;
        let alpha = (w0).sin() / (2.0 * q);

        let b0 = (1.0 - w0.cos()) / 2.0;
        let b1 = 1.0 - w0.cos();
        let b2 = (1.0 - w0.cos()) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * w0.cos();
        let a2 = 1.0 - alpha;

        // Normalisation
        BiquadCoeffs {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }
}
pub struct BiquadFilter {
    id: usize,
    pub coeffs: BiquadCoeffs,
    pub z1l: f32,
    pub z1r: f32,
    pub z2l: f32,
    pub z2r: f32,
}

impl BiquadFilter {
    pub fn new(frequency: f32, q: f32, id: usize) -> Self {
        BiquadFilter {
            coeffs: BiquadCoeffs::calc_biquad_coeffs(frequency, q),
            id: id,
            z1l: 0.0,
            z1r: 0.0,
            z2l: 0.0,
            z2r: 0.0,
        }
    }
}

impl EffectTrait for BiquadFilter {
    fn id(&self) -> usize {
        self.id
    }
    fn process(&mut self, input_sample_r: &mut f32, input_sample_l: &mut f32) {
        let output_sample_r = self.coeffs.b0 * *input_sample_r + self.z1r;
        let output_sample_l = self.coeffs.b0 * *input_sample_l + self.z1l;

        self.z1l = self.coeffs.b1 * *input_sample_l - self.coeffs.a1 * output_sample_l + self.z2l;
        self.z1r = self.coeffs.b1 * *input_sample_r - self.coeffs.a1 * output_sample_r + self.z2r;

        self.z2l = self.coeffs.b2 * *input_sample_l - self.coeffs.a2 * output_sample_l;
        self.z2r = self.coeffs.b2 * *input_sample_r - self.coeffs.a2 * output_sample_r;

        *input_sample_l = output_sample_l;
        *input_sample_r = output_sample_r;
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct EchoParams {
    pub delay: usize,
    pub feedback: f32,
    pub r_delay_offset: usize,
    pub l_delay_offset: usize,
    pub mix: Mix,
}

pub struct Echo {
    id: usize,
    pub delay: usize,
    pub feedback: f32,
    pub memory: MemoryBuffer,
    pub r_delay_offset: usize,
    pub l_delay_offset: usize,
    pub mix: Mix,
}

impl Echo {
    pub fn new(
        delay: usize,
        feedback: f32,
        r_delay_offset: usize,
        l_delay_offset: usize,
        mix: Mix,
        id: usize,
    ) -> Self {
        Echo {
            mix: mix,
            delay: delay,
            feedback: feedback.max(0.0).min(1.0),
            memory: MemoryBuffer::new(44100, 10.0),
            r_delay_offset: r_delay_offset,
            l_delay_offset: l_delay_offset,
            id: id,
        }
    }
}

impl EffectTrait for Echo {
    fn id(&self) -> usize {
        self.id
    }
    fn process(&mut self, input_l: &mut f32, input_r: &mut f32) {
        let l = self.memory.read_left(self.delay + self.l_delay_offset * 2);
        let r = self.memory.read_right(self.delay + self.r_delay_offset * 2);
        *input_l = self.mix.dry * *input_l + self.mix.wet * l * self.feedback;
        *input_r = self.mix.dry * *input_r + self.mix.wet * r * self.feedback;
        self.memory.write(*input_l, *input_r);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
