use std::path::Path;
use crate::synth::Synth;

#[derive(Debug)]
pub struct CompileError {
    pub kind: CompileErrorKind,
    pub position: Option<(usize, usize)>,
}

#[derive(Debug)]
pub enum CompileErrorKind {
    IOError(std::io::Error),
    TestError,
}

pub fn compile_file(path: impl AsRef<Path>) -> Result<Synth, CompileError> {
    Err(CompileError{
        kind: CompileErrorKind::TestError,
        position: None
    })
}
