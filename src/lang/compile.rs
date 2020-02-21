use crate::lang::parser::{ self, Node, CommandNode, ExpressionNode };
use crate::operator::Operator;
use crate::synth::{ self, Synth, Id, NodeKind };
use std::collections::HashMap;

#[derive(Debug)]
pub struct CompileError {
    pub kind: CompileErrorKind,
    pub pos: Option<(usize, usize)>
}

#[derive(Debug)]
pub enum CompileErrorKind {
    UnknownFunctionName(String),
    InvalidVariableName,
    InvalidNumberOfOperatorArgs,
    InvalidArgNumber,
    NoOutputVariables,
}

pub fn compile(nodes: Vec<Node<CommandNode>>) -> Result<(Synth, Id, Id), CompileError> {
    let mut synth = Synth::new();
    let mut variables = HashMap::new();
    let mut probes = Vec::new();

    for node in nodes.into_iter() {
        match node.kind {
            CommandNode::Assignment(name, expr) => {
                let id = compile_expression(*expr, &mut probes, &variables, &mut synth)?;
                variables.insert(name, id);
            }
        }
    }

    while let Some((probe_id, max_size, expr)) = probes.pop() {
        println!("{:?}", probe_id);
        println!("{:?}", max_size);
        println!("{:?}", expr);

        let expr_id = compile_expression(expr, &mut probes, &variables, &mut synth)?;
        synth.add_probe(probe_id, max_size, expr_id);
    }

    // Get the variables used for output. 
    // These are either 'out' for mono output,
    // or 'left' and 'right' for stereo
    let (left, right) = if let Some(&node) = variables.get("out") {
        (node, node)
    }else{
        let left = if let Some(&node) = variables.get("left") {
            node
        }else { return Err(CompileError {
                    kind: CompileErrorKind::NoOutputVariables,
                    pos: None });
        };
        let right = if let Some(&node) = variables.get("right") {
            node
        }else { return Err(CompileError {
                    kind: CompileErrorKind::NoOutputVariables,
                    pos: None });
        };

        (left, right)
    };

    Ok((synth, left, right))
}

fn compile_expression(expr: Node<ExpressionNode>, 
                      probes: &mut Vec<(Id, f32, Node<ExpressionNode>)>, 
                      vars: &HashMap<String, Id>, synth: &mut Synth)
                       -> Result<Id, CompileError> {
    match expr.kind {
        ExpressionNode::Float(value) => {
            let node_id = synth.add_node(synth::NodeKind::Constant(value), &[], &[]);
            Ok(synth.get_node_output(node_id).unwrap())
        },
        ExpressionNode::Variable(string) => {
            if let Some(&id) = vars.get(&string) {
                Ok(id)
            }else{
                Err(CompileError {
                    kind: CompileErrorKind::InvalidVariableName,
                    pos: expr.pos
                })
            }
        },
        ExpressionNode::Operator(op, args) => {
            if args.len() == 2 {
                let mut args = args.into_iter();
                let arg_1 = compile_expression(args.next().unwrap(), probes, vars, synth)?;
                let arg_2 = compile_expression(args.next().unwrap(), probes, vars, synth)?;
                let node_id = synth.add_node(NodeKind::ConstantOp(op), &[arg_1, arg_2], &[]);
                Ok(synth.get_node_output(node_id).unwrap())
            }else{
                Err(CompileError {
                    kind: CompileErrorKind::InvalidNumberOfOperatorArgs,
                    pos: args.get(0).map(|v| v.pos).flatten()
                })
            }
        },
        ExpressionNode::FunctionCall(name, const_args, args) => {
            match name.as_str() {
                "delay" => {
                    if args.len() == 2 {
                        let mut args = args.into_iter();
                        let time = compile_expression(args.next().unwrap(), probes, vars, synth)?;
                        let expr = args.next().unwrap();
                        let probe_id = synth.allocate_probe_id();
                        let max = if let Some(max) = const_args.get("max") { max.kind }
                                  else { 5.0 };
                        // NOTE: Support other samplerates than 48000 here!!!!!!!!
                        probes.push((probe_id, max, expr));
                        let node_id = synth.add_node(synth::NodeKind::Delay(max, probe_id), &[time], &[]);
                        Ok(synth.get_node_output(node_id).unwrap())
                    }else {
                        Err(CompileError {
                            kind: CompileErrorKind::InvalidArgNumber,
                            pos: args.get(0).map(|v| v.pos).flatten()
                        })
                    }
                },
                "clamp" => {
                    if args.len() == 1 {
                        let mut args = args.into_iter();
                        let arg_1 = compile_expression(args.next().unwrap(), probes, vars, synth)?;
                        let min = if let Some(min) = const_args.get("min") { min.kind }
                                  else { -1.0 };
                        let max = if let Some(max) = const_args.get("max") { max.kind }
                                  else { 1.0 };
                        println!("{} -> {}", min, max);
                        let node_id = synth.add_node(synth::NodeKind::Clamp(min, max), &[arg_1], &[]);
                        Ok(synth.get_node_output(node_id).unwrap())
                    }else {
                        Err(CompileError {
                            kind: CompileErrorKind::InvalidArgNumber,
                            pos: args.get(0).map(|v| v.pos).flatten()
                        })
                    }
                },
                "square" => {
                    if args.len() == 1 {
                        let mut args = args.into_iter();
                        let arg_1 = compile_expression(args.next().unwrap(), probes, vars, synth)?;
                        let offset = if let Some(off) = const_args.get("off") {
                            off.kind
                        }else{
                            0.0
                        };
                        let node_id = synth.add_node(synth::NodeKind::SquareOscillator, &[arg_1], &[offset]);
                        Ok(synth.get_node_output(node_id).unwrap())
                    }else{
                        Err(CompileError {
                            kind: CompileErrorKind::InvalidArgNumber,
                            pos: args.get(0).map(|v| v.pos).flatten()
                        })
                    }
                },
                "osc" => {
                    if args.len() == 1 {
                        let mut args = args.into_iter();
                        let arg_1 = compile_expression(args.next().unwrap(), probes, vars, synth)?;
                        let offset = if let Some(off) = const_args.get("off") {
                            off.kind
                        }else{
                            0.0
                        };

                        let node_id = synth.add_node(synth::NodeKind::Oscillator, &[arg_1], &[offset]);
                        Ok(synth.get_node_output(node_id).unwrap())
                    }else{
                        Err(CompileError {
                            kind: CompileErrorKind::InvalidArgNumber,
                            pos: args.get(0).map(|v| v.pos).flatten()
                        })
                    }
                },
                _ => {
                    Err(CompileError {
                        kind: CompileErrorKind::UnknownFunctionName(name),
                        pos: args.get(0).map(|v| v.pos).flatten()
                    })
                }
            }
        },
    }
}
