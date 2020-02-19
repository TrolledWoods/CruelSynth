use std::path::Path;
use crate::synth::Synth;
use crate::synth::NodeId;

mod tokenize;
mod parser;
mod compile;

#[derive(Debug)]
pub struct CompileError {
    pub kind: CompileErrorKind,
    pub pos: Option<(usize, usize)>,
}

#[derive(Debug)]
pub enum CompileErrorKind {
    IOError(std::io::Error),
    TokenizerError(tokenize::TokenErrorKind),
    ParseError(parser::ParseErrorKind),
    CompileError(compile::CompileErrorKind),
    TestError,
}

pub fn compile_file(path: impl AsRef<Path>) -> Result<(Synth, NodeId, NodeId), CompileError> {
    let contents = 
        std::fs::read_to_string(path)
        .map_err(|v| CompileError { kind: CompileErrorKind::IOError(v), pos: None })?;    

    let tokens = tokenize::tokenize(contents.as_str())
        .map_err(|v| CompileError { kind: CompileErrorKind::TokenizerError(v.kind), pos: Some(v.pos) })?;

    let commands = parser::parse_tokens(&mut tokens.into_iter().peekable())
        .map_err(|v| CompileError { kind: CompileErrorKind::ParseError(v.kind), pos: v.pos })?;

    let (synth, left, right) = compile::compile(commands)
        .map_err(|v| CompileError { kind: CompileErrorKind::CompileError(v.kind), pos: v.pos })?;

    Ok((synth, left, right))
}
