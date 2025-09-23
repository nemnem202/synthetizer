use crate::buffers::MemoryBuffer;

pub struct BiquadCoeffs {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
}

impl BiquadCoeffs {
    pub fn calc_biquad_coeffs(frequency: f32, q: f32, sample_rate: f32) -> BiquadCoeffs {
        let w0 = 2.0 * std::f32::consts::PI * frequency / sample_rate;
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
    pub coeffs: BiquadCoeffs,
    pub z1l: f32,
    pub z1r: f32,
    pub z2l: f32,
    pub z2r: f32,
}

impl BiquadFilter {
    pub fn process(&mut self, input_sample_r: f32, input_sample_l: f32) -> (f32, f32) {
        let output_sample_r = self.coeffs.b0 * input_sample_r + self.z1r;
        let output_sample_l = self.coeffs.b0 * input_sample_l + self.z1l;

        self.z1l = self.coeffs.b1 * input_sample_l - self.coeffs.a1 * output_sample_l + self.z2l;
        self.z1r = self.coeffs.b1 * input_sample_r - self.coeffs.a1 * output_sample_r + self.z2r;

        self.z2l = self.coeffs.b2 * input_sample_l - self.coeffs.a2 * output_sample_l;
        self.z2r = self.coeffs.b2 * input_sample_r - self.coeffs.a2 * output_sample_r;

        (output_sample_r, output_sample_l)
    }
}

pub struct Echo {
    pub delay: usize,
    pub feedback: f32,
    pub memory: MemoryBuffer,
    pub r_delay_offset: usize,
    pub l_delay_offset: usize,
}

impl Echo {
    pub fn new(delay: usize, feedback: f32, r_delay_offset: usize, l_delay_offset: usize) -> Self {
        Echo {
            delay: delay,
            feedback: feedback.max(0.0).min(1.0),
            memory: MemoryBuffer::new(44100, 10.0),
            r_delay_offset: r_delay_offset,
            l_delay_offset: l_delay_offset,
        }
    }

    pub fn process(&mut self, input_l: &mut f32, input_r: &mut f32) {
        let l = self.memory.read_left(self.delay + self.l_delay_offset * 2);
        let r = self.memory.read_right(self.delay + self.r_delay_offset * 2);
        *input_l += l * self.feedback;
        *input_r += r * self.feedback;
        self.memory.write(*input_l, *input_r);
    }
}
