use crate::{
    dsp_effects::{BiquadFilter, Echo, EffectTrait},
    toolkit::ToolKit,
    types::Mix,
};

pub struct Mixer {
    pub effects: Vec<Box<dyn EffectTrait>>,
}

impl Mixer {
    pub fn new() -> Self {
        let mut mixer = Self {
            effects: Vec::new(),
        };

        // Crée ton filtre
        let test_filter = BiquadFilter::new(800.0, 0.7, 5);

        let test_echo_mix = Mix { dry: 1.0, wet: 1.0 };

        let test_echo: Echo = Echo::new(
            test_echo_mix,
            ToolKit::convert_ms_to_sample(1000.0),
            0.7,
            ToolKit::convert_ms_to_sample(10.0),
            ToolKit::convert_ms_to_sample(50.0),
            10,
        );

        // Ajoute-le au vecteur (box pour dyn trait)
        mixer.effects.push(Box::new(test_filter));
        mixer.effects.push(Box::new(test_echo));

        mixer
    }
    pub fn render(&mut self, sample_l: &mut f32, sample_r: &mut f32) {
        for effect in &mut self.effects {
            effect.process(sample_l, sample_r);
        }
    }
}
