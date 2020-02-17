use crate::sound_gen::SoundGenerator;

#[inline]
fn unit_sin(t: f32) -> f32 {
    (2.0 * std::f32::consts::PI * t).sin()
}

#[derive(Clone)]
pub struct Oscillator {
    time: f32,
}

impl Oscillator {
    pub fn new() -> Oscillator {
        Oscillator {
            time: 0.0,
        }
    }
}

impl SoundGenerator for Oscillator {
    fn n_params(&self) -> usize { 1 }

    fn get_param_names(&self) -> &'static str {
        "frequency(hz)"
    }

    fn get_output(&mut self, params: &[f32], sample_rate: usize) -> f32 {
        self.time += params[0] / sample_rate as f32;
        unit_sin(self.time)
    }
}
