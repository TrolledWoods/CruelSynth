use crate::lang::parser::{ self, Node, CommandNode, ExpressionNode };
use crate::operator::Operator;
use crate::synth::{ Synth, Id, NodeKind };
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

    // Do the main compilation
    for node in nodes.into_iter() {
        // Match the different types of commands that can exist.
        match node.kind {
            CommandNode::Assignment(name, expr) => {
                // Create a node tree for the expression, 
                // then point a variable at it
                let id = compile_expression(*expr, &mut probes, &variables, &mut synth)?;
                variables.insert(name, id);
            }
        }
    }

    // Compile the probes
    while let Some((probe_id, max_size, expr)) = probes.pop() {
        // For each probe, we compile its node tree, i.e. the node it probes.
        // Since we compile these as a final step, they have access to every
        // variable defined in the file, which allows them to implement
        // cross referencing and referencing variables that are defined
        // after them, as in reality they are compiled after every variable
        // is added.
        let expr_id = compile_expression(expr, &mut probes, &variables, &mut synth)?;
        synth.add_probe(probe_id, max_size, expr_id);
    }

    // Get the variables used for output. 
    // These are either 'out' for mono output,
    // or 'left' and 'right' for stereo
    let (left, right) = match variables.get("out") {
        Some(&node) => (node, node),
        None => {
            let left = match variables.get("left") {
                Some(&node) => node,
                None => return Err(CompileError {
                            kind: CompileErrorKind::NoOutputVariables,
                            pos: None })
            };

            let right = match variables.get("right") {
                Some(&node) => node,
                None => return Err(CompileError {
                            kind: CompileErrorKind::NoOutputVariables,
                            pos: None })
            };

            (left, right)
        }
    };

    Ok((synth, left, right))
}

fn compile_expression(expr: Node<ExpressionNode>, 
                      probes: &mut Vec<(Id, f32, Node<ExpressionNode>)>, 
                      vars: &HashMap<String, Id>, synth: &mut Synth)
                       -> Result<Id, CompileError> {
    match expr.kind {
        ExpressionNode::Float(value) => {
            let node_id = synth.add_node(NodeKind::Constant(value), &[], &[]);
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

                        probes.push((probe_id, max, expr));
                        let node_id = synth.add_node(NodeKind::Delay(max, probe_id), &[time], &[]);
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
                        let node_id = synth.add_node(NodeKind::Clamp(min, max), &[arg_1], &[]);
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
                        let node_id = synth.add_node(NodeKind::SquareOscillator, &[arg_1], &[offset]);
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

                        let node_id = synth.add_node(NodeKind::Oscillator, &[arg_1], &[offset]);
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
