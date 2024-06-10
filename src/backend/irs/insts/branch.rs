use super::*;

#[derive(Clone)]
pub struct BranchInst(BranchOp, Reg, Reg, Label);
impl BranchInst {
    pub fn new(op: BranchOp, lhs: Reg, rhs: Reg, label: Label) -> Self {
        Self(op, lhs, rhs, label)
    }
    pub fn op(&self) -> &BranchOp {
        &self.0
    }
    pub fn lhs(&self) -> &Reg {
        &self.1
    }
    pub fn rhs(&self) -> &Reg {
        &self.2
    }
    pub fn label(&self) -> &Label {
        &self.3
    }
    pub fn lhs_mut(&mut self) -> &mut Reg {
        &mut self.1
    }
    pub fn rhs_mut(&mut self) -> &mut Reg {
        &mut self.2
    }
    pub fn label_mut(&mut self) -> &mut Label {
        &mut self.3
    }
    pub fn defs(&self) -> Vec<&Reg> {
        vec![]
    }
    pub fn uses(&self) -> Vec<&Reg> {
        vec![self.lhs(), self.rhs()]
    }
}

// beqz
// bgeu
// bgez
// bltu
// bltz
// bleu
// blez
// bgtu
// bgtz
#[derive(Clone)]
pub enum BranchOp {
    Beq,
    Bne,
    Blt,
    Ble,
    Bgt,
    Bge,
}

impl BranchOp {
    pub fn gen_asm(&self) -> String {
        match self {
            Self::Beq => String::from("beq"),
            Self::Bne => String::from("bne"),
            Self::Blt => String::from("blt"),
            Self::Ble => String::from("ble"),
            Self::Bgt => String::from("bgt"),
            Self::Bge => String::from("bge"),
        }
    }
}

impl BranchInst {
    pub fn gen_asm(&self) -> String {
        format!(
            "{} {},{},{}",
            self.0.gen_asm(),
            self.1.gen_asm(),
            self.2.gen_asm(),
            self.3.gen_asm(),
        )
    }
}