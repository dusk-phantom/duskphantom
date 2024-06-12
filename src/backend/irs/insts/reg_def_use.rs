use super::*;


/* def and impl RegUses and RegDefs */ 
pub trait RegUses {
    fn uses(&self) -> Vec<&Reg> {
        vec![]
    }
}
pub trait RegDefs {
    fn defs(&self) -> Vec<&Reg> {
        vec![]
    }
}

impl RegUses for Inst {
    fn uses(&self) -> Vec<&Reg> {
        match self {
            Inst::Add(inst) => inst.uses(),
            Inst::Sub(inst) => inst.uses(),
            Inst::Mul(inst) => inst.uses(),
            Inst::Rem(inst) => inst.uses(),
            Inst::Div(inst) => inst.uses(),
            Inst::SLL(inst) => inst.uses(),
            Inst::SRL(inst) => inst.uses(),
            Inst::Neg(inst) => inst.uses(),
            Inst::Mv(inst) => inst.uses(),
            Inst::Ld(inst) => inst.uses(),
            Inst::Sd(inst) => inst.uses(),
            Inst::Lw(inst) => inst.uses(),
            Inst::Sw(inst) => inst.uses(),
            Inst::La(inst) => inst.uses(),
            Inst::Jmp(inst) => inst.uses(),
            Inst::Branch(inst) => inst.uses(),
            Inst::Call(inst) => inst.uses(),
            Inst::SRA(inst) => inst.uses(),
            Inst::Ret => vec![],
            Inst::AND(inst) => inst.uses(),
            Inst::OR(inst) => inst.uses(),
            Inst::XOR(inst) => inst.uses(),
        }
    }
}
impl RegDefs for Inst {
    fn defs(&self) -> Vec<&Reg> {
        match self {
            Inst::Add(inst) => inst.defs(),
            Inst::Sub(inst) => inst.defs(),
            Inst::Mul(inst) => inst.defs(),
            Inst::Rem(inst) => inst.defs(),
            Inst::Div(inst) => inst.defs(),
            Inst::AND(inst) => inst.defs(),
            Inst::OR(inst) => inst.defs(),
            Inst::XOR(inst) =>inst.defs(),
            Inst::SLL(inst) => inst.defs(),
            Inst::SRL(inst) => inst.defs(),
            Inst::SRA(inst) => inst.defs(),
            Inst::Neg(inst) => inst.defs(),
            Inst::Mv(inst) => inst.defs(),
            Inst::Ld(inst) => inst.defs(),
            Inst::Sd(inst) => inst.defs(),
            Inst::Lw(inst) => inst.defs(),
            Inst::Sw(inst)=>inst.defs(),
            Inst::La(inst) => inst.defs(),
            Inst::Jmp(inst) => inst.defs(),
            Inst::Branch(inst) => inst.defs(),
            Inst::Call(inst) => inst.defs(),
            Inst::Ret => vec![],
            
        }
    }
}

impl RegUses for BranchInst {
    fn uses(&self) -> Vec<&Reg> {
        vec![self.lhs(), self.rhs()]
    }
}
impl RegDefs for BranchInst {
    fn defs(&self) -> Vec<&Reg> {
        vec![]
    }
}

impl RegUses for LaInst {}

impl RegDefs for LaInst {
    fn defs(&self) -> Vec<&Reg> {
        vec![self.dst()]
    }
}
impl RegUses for CallInst {}
impl RegDefs for CallInst {}
impl RegUses for JmpInst {}
impl RegDefs for JmpInst {}
impl RegUses for LdInst {
    fn uses(&self) -> Vec<&Reg> {
        vec![self.base()]
    }
}
impl RegDefs for LdInst {
    fn defs(&self) -> Vec<&Reg> {
        vec![self.dst()]
    }
}
impl RegUses for SdInst {
    fn uses(&self) -> Vec<&Reg> {
        vec![self.base(), self.dst()]
    }
}
impl RegDefs for SdInst {}
impl RegUses for LwInst {
    fn uses(&self) -> Vec<&Reg> {
        vec![self.base()]
    }
}
impl RegDefs for LwInst {
    fn defs(&self) -> Vec<&Reg> {
        vec![self.dst()]
    }
}

impl RegUses for SwInst {
    fn uses(&self) -> Vec<&Reg> {
        vec![self.base(), self.dst()]
    }
}
impl RegDefs for SwInst {}

