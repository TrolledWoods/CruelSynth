use crate::sound_gen::SoundGenerator;

impl<T: Fn(f32, f32) -> f32> SoundGenerator for T {
    fn n_params(&self) -> usize { 2 }
    fn get_param_names(&self) -> &'static str { "a, b" }
    fn get_output(&mut self, params: &[f32], _sample_rate: usize) -> f32 {
        self(params[0], params[1])
    }
}

#[derive(Clone)]
pub struct Delay {
    buffer: Vec<f32>,
    max_size: usize,
}

impl Delay {
    pub fn new(max_size: usize) -> Delay {
        Delay {
            buffer: Vec::with_capacity(max_size),
            max_size: max_size,
        }
    }
}

impl SoundGenerator for Delay {
    fn n_params(&self) -> usize { 2 }
    fn get_param_names(&self) -> &'static str { "input, delay time" }
    fn get_output(&mut self, params: &[f32], sample_rate: usize) -> f32 {
        self.buffer.insert(0, params[0]);
        if self.buffer.len() > self.max_size {
            self.buffer.pop();
        }
        let offset = (params[1] * sample_rate as f32) as usize;
        if let Some(val) = self.buffer.get(offset) {
            *val
        }else{
            0.0
        }
    }
}
