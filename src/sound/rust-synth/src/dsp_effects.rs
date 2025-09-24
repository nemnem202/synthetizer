use crate::{buffers::MemoryBuffer, constants::SAMPLE_RATE, types::Mix};

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
pub trait EffectTrait {
    fn id(&self) -> usize;
    fn process(&mut self, sample_l: &mut f32, sample_r: &mut f32);
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
}
