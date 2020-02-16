use crate::variables::Variables;

pub enum SampleGiver {
    Constant(f32),
    Generator(Box<dyn SoundGenerator>, Vec<SampleGiver>),
    Variable(usize),
}

impl SampleGiver {
    pub fn get_value(&mut self, vars: &Variables) -> f32 {
        match self {
            SampleGiver::Constant(constant) => *constant,
            SampleGiver::Variable(variable_id) => 
                vars.get_var_sample(variable_id)
                    .expect("Invalid variable id in SampleGiver, should not be possible"),
            SampleGiver::Generator(sound_gen, args) => {
                // We assume here that the number of arguments
                // is equal to the number of parameters in the
                // sound generator.
                for (i, arg) in args.iter_mut().enumerate() {
                    let output = arg.get_value(vars);
                    sound_gen.set_param(i, output);
                }

                sound_gen.get_output()
            }
        }
    }
}

pub trait SoundGenerator {
    /// Returns the number of parameters 
    /// This will never change.
    fn n_params(&self) -> usize;

    /// Panics if the param_id is larger than the
    /// number of parameters(n_params method)
    fn set_param(&mut self, param_id: usize, value: f32);

    /// Returns the output of the generator
    /// Can mutate internal state, so only call once
    /// per sampling point.
    fn get_output(&mut self) -> f32;
}
