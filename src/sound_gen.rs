pub enum SampleGiver {
    Constant(f32),
    Generator(Box<dyn SoundGenerator>, Vec<SampleGiver>),
    Variable(usize),
}

impl std::fmt::Debug for SampleGiver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SampleGiver::Constant(constant) => {
                write!(f, "const {}", constant)?;
            },
            SampleGiver::Generator(_, sample_givers) => {
                write!(f, "fn (")?;
                for (i, sample_giver) in sample_givers.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", {:?}", sample_giver)?;
                    }else{
                        write!(f, "{:?}", sample_giver)?;
                    }
                }
                write!(f, ")")?;
            },
            SampleGiver::Variable(id) => {
                write!(f, "var {}", id)?;
            }
        }

        Ok(())
    }
}

impl SampleGiver {
    pub fn get_value(&mut self, variables: &[f32], sample_rate: usize) -> f32 {
        match self {
            SampleGiver::Constant(constant) => *constant,
            SampleGiver::Variable(variable_id) => variables[*variable_id],
            SampleGiver::Generator(sound_gen, args) => {
                // We assume here that the number of arguments
                // is equal to the number of parameters in the
                // sound generator.
                let mut buffer = Vec::with_capacity(10);
                for arg in args.iter_mut() {
                    buffer.push(arg.get_value(variables, sample_rate));
                }

                sound_gen.get_output(&buffer[..], sample_rate)
            }
        }
    }
}

pub trait SoundGenerator {
    /// Returns the number of parameters 
    /// This will never change.
    fn n_params(&self) -> usize;

    fn get_param_names(&self) -> &'static str;

    /// Returns the output of the generator
    /// Can mutate internal state, so only call once
    /// per sampling point.
    /// The params argument has to be the same length
    /// as the n_params function returns
    fn get_output(&mut self, params: &[f32], sample_rate: usize) -> f32;
}
