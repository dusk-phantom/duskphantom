use crate::{impl_inst_from, impl_mem_inst, impl_three_op_inst, impl_two_op_inst, impl_unary_inst};

use super::*;

// 缺少浮点数的相关指令
// fabs.s
// fadd.s
// flw
// fsw
// fcvt.l.s
// fcvt.s.l
// fcvt.s.lu
// fcvt.s.w
// fcvt.s.wu
// fcvt.w.s
// fcvt.wu.s
// fdiv.s
// fle.d
// fle.s
// flt.s
// fmadd.s rd, rs1, rs2, rs3 // f[rd] = f[rs1] * f[rs2] + f[rs3]
// fmax.s
// fmin.s
// fmsub.s
// fmul.s
// fmv.s
// fmv.w.s
// fmv.x.w
// fneg.s
// fnmadd.s  rd, rs1, rs2, rs3 // f[rd] = -(f[rs1] * f[rs2] + f[rs3])
// fnmsub.s  rd, rs1, rs2, rs3 // f[rd] = -(f[rs1] * f[rs2] - f[rs3])
// fsub.s

#[derive(Clone)]
pub enum Inst {
    // 拓展
    // sext.w

    // li
    // algebraic operation
    Add(AddInst),

    // xor
    // xori

    // subw
    Sub(SubInst),

    Mul(MulInst),
    Div(DivInst),
    Rem(RemInst),

    SLL(SllInst),
    SRL(SrlInst),
    Neg(NegInst),
    // data transfer operation
    Mv(MvInst),
    Ld(LdInst),
    Sd(SdInst),
    La(LaInst),
    // control flow operation
    Jmp(JmpInst),
    Branch(BranchInst),
    Call(CallInst),
    // tail 尾调用
    Ret,
}

// addi
// addiw
// addw
// and
// andi

impl_three_op_inst!(AddInst, "add");
impl_three_op_inst!(SubInst, "sub");
impl_three_op_inst!(MulInst, "mul");
impl_three_op_inst!(RemInst, "rem");
impl_three_op_inst!(DivInst, "div");
impl_three_op_inst!(SllInst, "sll");
impl_three_op_inst!(SrlInst, "srl");
impl_two_op_inst!(NegInst);
impl_two_op_inst!(MvInst);
impl_mem_inst!(LdInst, "ld");
impl_mem_inst!(SdInst, "sd");
impl_mem_inst!(SwInst, "sw");
impl_mem_inst!(LwInst, "lw");

impl_unary_inst!(JmpInst, "j");
impl_unary_inst!(CallInst, "call");

// slli
// slliw
// sllw
// sra 算术右移
// srai
// sraiw
// sraw
// srli
// srliw
// srlw
// ori
// or
// not
// negw

// lw

// lla
#[derive(Clone)]
pub struct LaInst(Reg, Label);
impl LaInst {
    pub fn new(dst: Reg, label: Label) -> Self {
        Self(dst, label)
    }
    pub fn dst(&self) -> &Reg {
        &self.0
    }
    pub fn label(&self) -> &Label {
        &self.1
    }
    pub fn dst_mut(&mut self) -> &mut Reg {
        &mut self.0
    }
    pub fn label_mut(&mut self) -> &mut Label {
        &mut self.1
    }
}

// j
// jal
// jalr
// jr

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

impl LaInst {
    pub fn gen_asm(&self) -> String {
        format!("la {},{}", self.0.gen_asm(), self.1.gen_asm())
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

impl Inst {
    pub fn gen_asm(&self) -> String {
        match self {
            Inst::Add(inst) => inst.gen_asm(),
            Inst::Sub(inst) => inst.gen_asm(),
            Inst::Mul(inst) => inst.gen_asm(),
            Inst::Rem(inst) => inst.gen_asm(),
            Inst::Div(inst) => inst.gen_asm(),
            Inst::SLL(inst) => inst.gen_asm(),
            Inst::SRL(inst) => inst.gen_asm(),
            Inst::Neg(inst) => inst.gen_asm(),
            Inst::Mv(inst) => inst.gen_asm(),
            Inst::Ld(inst) => inst.gen_asm(),
            Inst::Sd(inst) => inst.gen_asm(),
            Inst::La(inst) => inst.gen_asm(),
            Inst::Jmp(inst) => inst.gen_asm(),
            Inst::Branch(inst) => inst.gen_asm(),
            Inst::Call(inst) => inst.gen_asm(),
            Inst::Ret => String::from("ret"),
        }
    }
}

// impl From<T> for Inst
impl_inst_from!(SdInst, Sd);
impl_inst_from!(LdInst, Ld);
impl_inst_from!(AddInst, Add);
impl_inst_from!(SubInst, Sub);
impl_inst_from!(MulInst, Mul);
impl_inst_from!(RemInst, Rem);
impl_inst_from!(DivInst, Div);
impl_inst_from!(SllInst, SLL);
impl_inst_from!(SrlInst, SRL);
impl_inst_from!(NegInst, Neg);
impl_inst_from!(MvInst, Mv);
impl_inst_from!(JmpInst, Jmp);
impl_inst_from!(BranchInst, Branch);
impl_inst_from!(CallInst, Call);
impl_inst_from!(LaInst, La);

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
            Inst::La(inst) => inst.uses(),
            Inst::Jmp(inst) => inst.uses(),
            Inst::Branch(inst) => inst.uses(),
            Inst::Call(inst) => inst.uses(),
            Inst::Ret => vec![],
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
            Inst::SLL(inst) => inst.defs(),
            Inst::SRL(inst) => inst.defs(),
            Inst::Neg(inst) => inst.defs(),
            Inst::Mv(inst) => inst.defs(),
            Inst::Ld(inst) => inst.defs(),
            Inst::Sd(inst) => inst.defs(),
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
