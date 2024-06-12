
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
            Inst::Sll(inst) => inst.uses(),
            Inst::Srl(inst) => inst.uses(),
            Inst::Neg(inst) => inst.uses(),
            Inst::Slt(inst) => inst.uses(),
            Inst::Mv(inst) => inst.uses(),
            Inst::Ld(inst) => inst.uses(),
            Inst::Sd(inst) => inst.uses(),
            Inst::Lw(inst) => inst.uses(),
            Inst::Sw(inst) => inst.uses(),
            Inst::Load(inst) => inst.uses(),
            Inst::Store(inst) => inst.uses(),
            Inst::La(inst) => inst.uses(),
            Inst::Jmp(inst) => inst.uses(),
            Inst::Branch(inst) => inst.uses(),
            Inst::Call(inst) => inst.uses(),
            Inst::SRA(inst) => inst.uses(),
            Inst::And(inst) => inst.uses(),
            Inst::Or(inst) => inst.uses(),
            Inst::Xor(inst) => inst.uses(),
            Inst::Ret => vec![],
            Inst::Tail(_) =>vec![],
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
            Inst::And(inst) => inst.defs(),
            Inst::Or(inst) => inst.defs(),
            Inst::Xor(inst) =>inst.defs(),
            Inst::Sll(inst) => inst.defs(),
            Inst::Srl(inst) => inst.defs(),
            Inst::SRA(inst) => inst.defs(),
            Inst::Neg(inst) => inst.defs(),
            Inst::Mv(inst) => inst.defs(),
            Inst::Ld(inst) => inst.defs(),
            Inst::Sd(inst) => inst.defs(),
            Inst::Load(inst) => inst.defs(),
            Inst::Store(inst) => inst.defs(),
            Inst::Lw(inst) => inst.defs(),
            Inst::Sw(inst)=>inst.defs(),
            Inst::La(inst) => inst.defs(),
            Inst::Jmp(inst) => inst.defs(),
            Inst::Branch(inst) => inst.defs(),
            Inst::Call(inst) => inst.defs(),
            Inst::Ret => vec![],
            Inst::Tail(_) =>vec![],
            Inst::Slt(inst) => inst.defs(),
            
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

impl RegDefs for LoadInst{}
impl RegUses for LoadInst{}
impl RegDefs for StoreInst{}
impl RegUses for StoreInst{}