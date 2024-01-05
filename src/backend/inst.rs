use super::*;

#[derive(Clone)]
pub enum Operand {
    Reg(Reg),
    Imm(Imm),
    Fmm(Fmm),
    Label(Label),
}

pub type Reg = u64;
pub type Imm = i64;
pub type Fmm = f64;
pub type Label = String;

impl Operand {
    pub fn reg(&self) -> Option<Reg> {
        match self {
            Self::Reg(reg) => Some(*reg),
            _ => None,
        }
    }
    pub fn imm(&self) -> Option<Imm> {
        match self {
            Self::Imm(imm) => Some(*imm),
            _ => None,
        }
    }
    pub fn fmm(&self) -> Option<Fmm> {
        match self {
            Self::Fmm(fmm) => Some(*fmm),
            _ => None,
        }
    }
    pub fn label(&self) -> Option<Label> {
        match self {
            Self::Label(label) => Some(label.clone()),
            _ => None,
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
