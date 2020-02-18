use crate::lang::parser::{ self, Node, CommandNode, ExpressionNode };
use crate::operator::Operator;
use crate::synth::{ self, Synth, NodeId };
use std::collections::HashMap;

#[derive(Debug)]
pub struct CompileError {
    pub kind: CompileErrorKind,
    pub pos: Option<(usize, usize)>
}

#[derive(Debug)]
pub enum CompileErrorKind {
    UnknownFunctionName,
    InvalidVariableName,
    InvalidNumberOfOperatorArgs,
    InvalidArgNumber,
}

pub fn compile(nodes: Vec<Node<CommandNode>>) -> Result<Synth, CompileError> {
    let mut synth = Synth::new();
    let mut variables = HashMap::new();

    for node in nodes.into_iter() {
        match node.kind {
            CommandNode::Assignment(name, expr) => {
                let id = compile_expression(*expr, &variables, &mut synth)?;
                variables.insert(name, id);
            }
        }
    }

    Ok(synth)
}

fn compile_expression(expr: Node<ExpressionNode>, vars: &HashMap<String, NodeId>, synth: &mut Synth)
        -> Result<NodeId, CompileError> {
    match expr.kind {
        ExpressionNode::Float(value) => {
            Ok(synth.add_node(synth::Node::constant(value)))
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
                let arg_1 = compile_expression(args.next().unwrap(), vars, synth)?;
                let arg_2 = compile_expression(args.next().unwrap(), vars, synth)?;
                Ok(synth.add_node(synth::Node::constant_op(op, arg_1, arg_2)))
            }else{
                Err(CompileError {
                    kind: CompileErrorKind::InvalidNumberOfOperatorArgs,
                    pos: args.get(0).map(|v| v.pos).flatten()
                })
            }
        },
        ExpressionNode::FunctionCall(name, const_args, args) => {
            match name.as_str() {
                "osc" => {
                    if args.len() == 1 {
                        let mut args = args.into_iter();
                        let arg_1 = compile_expression(args.next().unwrap(), vars, synth)?;
                        let offset = if let Some(off) = const_args.get("off") {
                            off.kind
                        }else{
                            0.0
                        };
                        Ok(synth.add_node(synth::Node::oscillator(arg_1, offset)))
                    }else{
                        Err(CompileError {
                            kind: CompileErrorKind::InvalidArgNumber,
                            pos: args.get(0).map(|v| v.pos).flatten()
                        })
                    }
                },
                _ => {
                    Err(CompileError {
                        kind: CompileErrorKind::UnknownFunctionName,
                        pos: args.get(0).map(|v| v.pos).flatten()
                    })
                }
            }
        },
    }
}
