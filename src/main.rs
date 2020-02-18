use std::path::Path;

mod synth;
mod lang;

fn main() {
    use synth::{ Node, NodeType, ConstantOp };

    let mut synth = synth::Synth::new();
    let const_1 = synth.add_node(Node::constant(10.0));
    let yes = synth.add_node(Node::oscillator(const_1, 0.0));
    let const_2 = synth.add_node(Node::constant(120.0));
    let const_3 = synth.add_node(Node::constant(150.0));
    let const_4 = synth.add_node(Node::constant(1.25));
    let osc_yes = synth.add_node(Node::oscillator(const_4, 0.0));
    let osc_mul = synth.add_node(Node::constant_op(ConstantOp::Mult, osc_yes, const_2));
    let mult = synth.add_node(Node::constant_op(ConstantOp::Mult, osc_mul, yes));
    let add = synth.add_node(Node::constant_op(ConstantOp::Add, const_3, mult));
    let osc = synth.add_node(Node::oscillator(add, 0.0));

    println!("{:?}", &synth);

    let mut samples = Vec::new();
    let mut buffer = Vec::new();
    for i in (0..(48000 * 10)) {
        synth.run(&mut buffer, 1.0 / 48000.0);
        samples.push((buffer[osc.0 as usize], buffer[osc.0 as usize]));
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
