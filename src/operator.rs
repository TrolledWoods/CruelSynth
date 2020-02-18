#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Operator {
    Add,
    Sub,
    Mult,
    Div,
    Mod,
}

impl Operator {
    #[inline]
    pub fn evaluate(&self, a: f32, b: f32) -> f32 {
        use Operator::*;
        match self {
            Add => a + b,
            Sub => a - b,
            Mult => a * b,
            Div => a / b,
            Mod => a % b,
        }
    }
}
