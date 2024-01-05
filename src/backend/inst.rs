use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

use super::*;

#[derive(Clone)]
pub enum Operand {
    Reg(Reg),
    Imm(Imm),
    Fmm(Fmm),
    Label(Label),
}
#[derive(Clone)]
pub struct Reg(u64);
#[derive(Clone)]
pub struct Imm(i64);
#[derive(Clone)]
pub struct Fmm(f64);
#[derive(Clone)]
pub struct Label(String);

// 通用寄存器
const REG_ZERO: Reg = Reg(0);
const REG_RA: Reg = Reg(1);
const REG_SP: Reg = Reg(2);
const REG_GP: Reg = Reg(3);
const REG_TP: Reg = Reg(4);
const REG_T0: Reg = Reg(5);
const REG_T1: Reg = Reg(6);
const REG_T2: Reg = Reg(7);
const REG_S0: Reg = Reg(8); //栈帧寄存器
const REG_S1: Reg = Reg(9); //保留寄存器
const REG_A0: Reg = Reg(10); //返回值寄存器 以及 函数参数寄存器
const REG_A1: Reg = Reg(11);
const REG_A2: Reg = Reg(12);
const REG_A3: Reg = Reg(13);
const REG_A4: Reg = Reg(14);
const REG_A5: Reg = Reg(15);
const REG_A6: Reg = Reg(16);
const REG_A7: Reg = Reg(17);
const REG_S2: Reg = Reg(18);
const REG_S3: Reg = Reg(19);
const REG_S4: Reg = Reg(20);
const REG_S5: Reg = Reg(21);
const REG_S6: Reg = Reg(22);
const REG_S7: Reg = Reg(23);
const REG_S8: Reg = Reg(24);
const REG_S9: Reg = Reg(25);
const REG_S10: Reg = Reg(26);
const REG_S11: Reg = Reg(27);
const REG_T3: Reg = Reg(28);
const REG_T4: Reg = Reg(29);
const REG_T5: Reg = Reg(30);
const REG_T6: Reg = Reg(31);
// 浮点寄存器

// 虚拟寄存器计数器
lazy_static! {
    static ref VIRTUAL_REG_COUNTER: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
}

impl Reg {
    #[inline]
    pub fn to_str(&self) -> &'static str {
        match self.0 {
            1 => "ra",
            2 => "sp",
            3 => "fp",
            4 => "gp",
            _ => panic!("unknown register"),
        }
    }
    #[inline]
    pub fn gen_asm(&self) -> String {
        self.to_str().to_string()
    }
    #[inline]
    pub fn is_phisic(&self) -> bool {
        match self.0 {
            0..=31 => true,
            _ => false,
        }
    }

    #[inline]
    pub fn gen_virtual_reg() -> Self {
        let mut counter = VIRTUAL_REG_COUNTER.lock().unwrap();
        let reg = Reg(*counter);
        *counter += 1;
        reg
    }
}
impl Imm {
    pub fn gen_asm(&self) -> String {
        format!("{}", self.0)
    }
}
impl Fmm {
    pub fn gen_asm(&self) -> String {
        format!("{}", self.0)
    }
}
impl Label {
    pub fn gen_asm(&self) -> String {
        format!("{}", self.0)
    }
}

impl Operand {
    pub fn reg(&self) -> Option<Reg> {
        match self {
            Self::Reg(reg) => Some(reg.clone()),
            _ => None,
        }
    }
    pub fn imm(&self) -> Option<Imm> {
        match self {
            Self::Imm(imm) => Some(imm.clone()),
            _ => None,
        }
    }
    pub fn fmm(&self) -> Option<Fmm> {
        match self {
            Self::Fmm(fmm) => Some(fmm.clone()),
            _ => None,
        }
    }
    pub fn label(&self) -> Option<Label> {
        match self {
            Self::Label(label) => Some(label.clone()),
            _ => None,
        }
    }

    pub fn gen_asm(&self) -> String {
        match self {
            Self::Reg(reg) => reg.gen_asm(),
            Self::Imm(imm) => imm.gen_asm(),
            Self::Fmm(fmm) => fmm.gen_asm(),
            Self::Label(label) => label.gen_asm(),
        }
    }
}

pub enum Inst {
    // 运算类型指令
    Add(AddInst),
    Mul(MulInst),
    Div(DivInst),
    // 数据移动指令
    Mv(MvInst),
    Ld(LdInst),
    Sd(SdInst),
    La(LaInst),
    // 控制流指令
    Jmp(JmpInst),
    Branch(BranchInst),
    Call(CallInst),
    Ret,
}

pub struct OneOpInst(Operand);
pub struct TwoOpInst(Operand, Operand);
pub struct ThreeOpInst(Operand, Operand, Operand);

impl OneOpInst {
    pub fn gen_asm(&self) -> String {
        // TODO
        String::new()
    }
}
impl TwoOpInst {
    pub fn gen_asm(&self) -> String {
        // TODO
        String::new()
    }
}
impl ThreeOpInst {
    pub fn gen_asm(&self) -> String {
        // TODO
        String::new()
    }
}

type AddInst = ThreeOpInst;
type MulInst = ThreeOpInst;
type DivInst = ThreeOpInst;
type MvInst = TwoOpInst;
type LdInst = TwoOpInst;
type SdInst = TwoOpInst;
type LaInst = TwoOpInst;
type JmpInst = OneOpInst;
type BranchInst = ThreeOpInst;
type CallInst = OneOpInst;

impl AddInst {}
impl MulInst {}
impl DivInst {}
impl MvInst {}
impl LdInst {}
impl SdInst {}
impl LaInst {}
impl JmpInst {}
impl BranchInst {}
impl CallInst {}

impl Inst {
    pub fn gen_asm(&self) -> String {
        match self {
            Inst::Add(inst) => inst.gen_asm(),
            Inst::Mul(inst) => inst.gen_asm(),
            Inst::Div(inst) => inst.gen_asm(),
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

// unit test
#[cfg(test)]
pub mod tests {
    use super::*;
}
