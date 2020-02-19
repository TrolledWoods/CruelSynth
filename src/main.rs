use std::path::Path;

mod synth;
mod lang;
mod operator;

fn main() {
    use synth::{ Node, NodeType };
    use operator::Operator;

    let (mut synth, left_id, right_id) = lang::compile_file("input.txt").unwrap();

    let mut samples = Vec::new();
    let mut buffer = Vec::new();
    let mut per_frame = 1.0 / 48000.0;
    for _ in (0..(48000.0 * 100.0) as usize) {
        synth.run(&mut buffer, per_frame);
        samples.push((buffer[left_id.0 as usize], buffer[right_id.0 as usize]));
    }

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
