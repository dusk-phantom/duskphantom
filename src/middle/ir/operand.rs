use super::*;

#[derive(Clone)]
pub enum Operand {
    Constant(Constant),

    Global(GlobalPtr),
    Parametr(Parameter),

    Instruction(InstPtr),
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Constant(c) => write!(f, "{}", c),
            Operand::Global(g) => write!(f, "{}", g),
            Operand::Parametr(p) => write!(f, "{}", p),
            Operand::Instruction(inst) => write!(f, "{}", inst),
        }
    }
}

impl From<Constant> for Operand {
    fn from(c: Constant) -> Self {
        Self::Constant(c)
    }
}

impl From<InstPtr> for Operand {
    fn from(inst: InstPtr) -> Self {
        Self::Instruction(inst)
    }
}

impl From<Parameter> for Operand {
    fn from(param: Parameter) -> Self {
        Self::Parametr(param)
    }
}
