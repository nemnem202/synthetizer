use web_sys::console;

use crate::{
    dsp_effects::{BiquadFilter, Echo, EchoParams, EffectTrait},
    toolkit::ToolKit,
    types::Mix,
};

pub struct Mixer {
    pub effects: Vec<Box<dyn EffectTrait>>,
    pub ECHO_DEFAULT_PRESET: EchoParams,
}

impl Mixer {
    pub fn new() -> Self {
        let mut mixer = Self {
            effects: Vec::new(),
            ECHO_DEFAULT_PRESET: EchoParams {
                delay: ToolKit::convert_ms_to_sample(1000.0),
                feedback: 0.7,
                l_delay_offset: ToolKit::convert_ms_to_sample(10.0),
                r_delay_offset: ToolKit::convert_ms_to_sample(50.0),
                mix: Mix { dry: 1.0, wet: 0.7 },
            },
        };

        // Crée ton filtre
        let test_filter = BiquadFilter::new(800.0, 0.7, 5);

        // let test_echo_mix = Mix { dry: 1.0, wet: 1.0 };

        // let test_echo: Echo = Echo::new(
        //     test_echo_mix,
        //     ToolKit::convert_ms_to_sample(1000.0),
        //     0.7,
        //     ToolKit::convert_ms_to_sample(10.0),
        //     ToolKit::convert_ms_to_sample(50.0),
        //     10,
        // );

        // Ajoute-le au vecteur (box pour dyn trait)
        mixer.effects.push(Box::new(test_filter));
        // mixer.effects.push(Box::new(test_echo));

        mixer
    }
    pub fn render(&mut self, sample_l: &mut f32, sample_r: &mut f32) {
        for effect in &mut self.effects {
            effect.process(sample_l, sample_r);
        }
    }

    pub fn create_echo(&mut self, id: u32, params: Vec<f32>) {
        console::log_1(&format!("Echo créé à l'id {}", id).into());
        let echo = Echo::new(
            self.ECHO_DEFAULT_PRESET.delay,
            self.ECHO_DEFAULT_PRESET.feedback,
            self.ECHO_DEFAULT_PRESET.r_delay_offset,
            self.ECHO_DEFAULT_PRESET.l_delay_offset,
            self.ECHO_DEFAULT_PRESET.mix,
            id as usize,
        );

        self.effects.push(Box::new(echo));
    }

    pub fn update_echo(&mut self, id: u32, params: Vec<f32>) {
        // let mix = Mix {
        //     dry: params[5],
        //     wet: params[6],
        // };
        // let echo = Echo::new(
        //     params[1] as usize,
        //     params[2],
        //     params[3] as usize,
        //     params[4] as usize,
        //     mix,
        //     id as usize,
        // );
    }
}
