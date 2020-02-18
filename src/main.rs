use std::path::Path;
use std::collections::HashMap;

mod sound_gen;
mod variables;
mod oscillator;
mod effects;
mod synth;

use oscillator::Oscillator;
use sound_gen::SampleGiver;
use sound_gen::SoundGenerator;
use variables::Variables;

fn update_synthesizer(variables: &mut Variables, sample_rate: usize, old_sample_cache: &[f32], sample_cache_output: &mut [f32]) {
    assert!(variables.is_verified(), "A synthesizer has to have verified variables");
    assert_eq!(variables.len(), old_sample_cache.len());
    assert_eq!(variables.len(), old_sample_cache.len());

    for (i, generator) in variables.data.iter_mut().map(|v| v.as_mut().unwrap()).enumerate() {
        sample_cache_output[i] = generator.get_value(old_sample_cache, sample_rate);
    }
}

fn run_synthesizer(synthesizer: &mut Variables) -> Result<Vec<(f32, f32)>, String> {
    if !synthesizer.verify() {
        return Err(format!("The synthesizer is not valid!"));
    }

    let left_id = synthesizer.name_to_id("left").ok_or_else(|| format!("The 'left' channel is not defined"))?;
    let right_id = synthesizer.name_to_id("right").ok_or_else(|| format!("The 'right' channel is not defined"))?;

    let mut buf_a = vec![0.0; synthesizer.len()];
    let mut buf_b = vec![0.0; synthesizer.len()];
    let mut swap = false;

    let mut samples = Vec::new();
    for i in 0..(48000 * 10) {
        if i % 23467 == 0 {
            println!("{:?}", &buf_a);
        }
        if swap {
            update_synthesizer(synthesizer, 48000, &buf_a, &mut buf_b);
            samples.push((buf_a[left_id], buf_a[right_id]));
        }else {
            update_synthesizer(synthesizer, 48000, &buf_b, &mut buf_a);
            samples.push((buf_b[left_id], buf_b[right_id]));
        }

        swap = !swap;
    }

    Ok(samples)
}

type Functions = HashMap<&'static str, Box<dyn Fn() -> Box<dyn SoundGenerator>>>;

fn parse_expression<'a>(input: &mut impl Iterator<Item = &'a str>, variables: &mut Variables, functions: &Functions) -> Result<SampleGiver, String> {
    let property = input.next().ok_or_else(|| format!("Expected a function name or something"))?;

    if property == "(" || property == ")" {
        // Temporary system to allow for some paranthesees
        parse_expression(input, variables, functions)

    // The length should be at least 1, therefore this should be fine
    }else if &property[0..1] == "$" {
        // It's a variable
        let variable_name = &property[1..];
        if variable_name.len() == 0 {
            return Err(format!("A variable was expected after a '$'"));
        }

        Ok(SampleGiver::Variable(variables.name_to_id_or_add(variable_name)))
    }else {
        let parsed: Option<f32> = property.parse::<f32>().ok();
        if let Some(value) = parsed {
            // It's a constant
            Ok(SampleGiver::Constant(value))
        }else{
            // It's a function!
            if let Some(function) = functions.get(property) {
                let function = function();
                let n_params = function.n_params();
                
                // Get each parameter
                let mut params = Vec::with_capacity(n_params);
                for param in 0..n_params {
                    let expression = parse_expression(input, variables, functions)?;
                    params.push(expression);
                }

                Ok(SampleGiver::Generator(function, params))
            }else{
                Err(format!("Invalid function name '{}'", property))
            }
        }
    }
}

fn parse_synthesizer(input: &str, functions: &Functions) -> Result<Variables, String> {
    let mut variables = Variables::new();
    for line in input.lines().map(|v| v.trim()).filter(|v| v.len() > 0) {
        let mut words = line.split_whitespace();
        let name = words.next().unwrap();
        if name == "#" { continue; }

        let assignment = words.next().ok_or_else(|| format!("Expected '='"))?;
        if assignment != "=" {
            return Err(format!("Expected '='"));
        }

        let contents = parse_expression(&mut words, &mut variables, functions)?;
        variables.add_var(name.to_string(), contents);
    }
    Ok(variables)
}

fn main() {
    use synth::{ ShallowNode, NodeType, ConstantOp };

    let mut synth = synth::Synth::new();
    let const_1 = synth.add_shallow_node(ShallowNode::constant(50.0));
    let osc = synth.add_shallow_node(ShallowNode::oscillator(const_1, 0.0));

    return;

    let mut functions: HashMap<&'static str, Box<dyn Fn() -> Box<dyn SoundGenerator>>> = HashMap::new();
    functions.insert("osc", Box::new(|| Box::new(Oscillator::new())));
    functions.insert("*", Box::new(|| Box::new(|a, b| a * b)));
    functions.insert("+", Box::new(|| Box::new(|a, b| a + b)));
    functions.insert("clamp", Box::new(|| Box::new(|a: f32, b: f32| a.min(1.0).max(-1.0))));
    functions.insert("delay", Box::new(|| Box::new(effects::Delay::new(80000))));
    functions.insert("big_delay", Box::new(|| Box::new(effects::Delay::new(180000))));

    let program = std::fs::read_to_string("input.txt").unwrap();

    let mut synthesizer = parse_synthesizer(&program[..], &functions).unwrap();

    for (i, var) in synthesizer.data.iter().enumerate() {
        println!("{}: {:?}", i, &var);
    }

    let samples = run_synthesizer(&mut synthesizer).unwrap();

    write_to_wave("C:/Users/johnm/Music/test2.wav", &samples[..], 48000);
}

fn write_to_wave(path: impl AsRef<Path>, data: &[(f32, f32)], sample_rate: u32) {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float
    };

    let mut writer = hound::WavWriter::create(path, spec).unwrap();
    for (left, right) in data {
        writer.write_sample(left.min(1.0).max(-1.0)).unwrap();
        writer.write_sample(right.min(1.0).max(-1.0)).unwrap();
    }
}
