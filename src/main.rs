use std::path::Path;

mod synth;
mod lang;
mod operator;

fn main() {
    use std::env;
    use std::path::{ Path, PathBuf };

    let mut args = env::args();
    args.next();
    if let Some(path) = args.next() {
        println!("How long should the generated file be?");
        let time = get_number();

        let mut path = PathBuf::from(path);

        use operator::Operator;

        let (synth, left_id, right_id) = lang::compile_file(&path).unwrap();
        let mut executor = synth::ExecutionData::new(&synth, 48000);

        let mut samples = Vec::new();
        for i in (0..(48000.0 * time) as usize) {
            executor.run();
            samples.push((executor.get_data(left_id).unwrap(), executor.get_data(right_id).unwrap()));
        }

        path.set_extension("wav");
        write_to_wave(&path, &samples[..], 48000);
    }
}

fn get_number() -> f32 {
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();
    buffer.trim().parse().expect("Please enter a number!")
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
