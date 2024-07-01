use super::*;

#[derive(Clone, PartialEq, Debug)]
pub enum Operand {
    Constant(Constant),

    Global(GlobalPtr),
    Parameter(ParaPtr),

    Instruction(InstPtr),
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Constant(c) => write!(f, "{}", c),
            Operand::Global(g) => write!(f, "{}", g),
            Operand::Parameter(p) => write!(f, "{}", p),
            Operand::Instruction(inst) => write!(f, "{}", inst),
        }
    }
}

impl Operand {
    pub fn get_type(&self) -> ValueType {
        match self {
            Operand::Constant(c) => c.get_type(),
            // Type of global var identifier (@gvar) is pointer
            Operand::Global(g) => ValueType::Pointer(g.value_type.clone().into()),
            Operand::Parameter(p) => p.value_type.clone(),
            Operand::Instruction(inst) => inst.get_value_type(),
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

impl From<ParaPtr> for Operand {
    fn from(param: ParaPtr) -> Self {
        Self::Parameter(param)
    }
}

impl From<GlobalPtr> for Operand {
    fn from(gvar: GlobalPtr) -> Self {
        Self::Global(gvar)
    }
}
