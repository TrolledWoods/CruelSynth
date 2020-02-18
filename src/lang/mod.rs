use std::path::Path;
use crate::synth::Synth;

mod tokenize;

#[derive(Debug)]
pub struct CompileError {
    pub kind: CompileErrorKind,
    pub pos: Option<(usize, usize)>,
}

#[derive(Debug)]
pub enum CompileErrorKind {
    IOError(std::io::Error),
    TestError,
}

pub fn compile_file(path: impl AsRef<Path>) -> Result<Synth, CompileError> {
    let contents = 
        std::fs::read_to_string(path)
        .map_err(|v| CompileError { kind: CompileErrorKind::IOError(v), pos: None })?;    

    tokenize::print_tokens(&tokenize::tokenize(contents.as_str()).unwrap()[..]);

    Err(CompileError {
        kind: CompileErrorKind::TestError,
        pos: None,
    })
}
