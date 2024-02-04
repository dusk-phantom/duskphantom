use crate::utils::paral_counter::ParalCounter;
use once_cell::sync::Lazy;

#[derive(Clone)]
pub enum Operand {
    Reg(Reg),
    Imm(Imm),
    Fmm(Fmm),
    Label(Label),
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Reg {
    id: u32,
    is_usual: bool,
}
impl Reg {
    const fn new(id: u32, is_usual: bool) -> Self {
        Self { id, is_usual }
    }
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn is_usual(&self) -> bool {
        self.is_usual
    }
}
#[derive(Clone)]
pub struct Imm(i64);
#[derive(Clone)]
pub struct Fmm(f64);
#[derive(Clone)]
pub struct Label(String);

// 基于 Reg::new(id, is_usual) 的寄存器定义重写如上代码
pub const REG_ZERO: Reg = Reg::new(0, true);
pub const REG_RA: Reg = Reg::new(1, true);
pub const REG_SP: Reg = Reg::new(2, true);
pub const REG_GP: Reg = Reg::new(3, true);
pub const REG_TP: Reg = Reg::new(4, true);
pub const REG_T0: Reg = Reg::new(5, true);
pub const REG_T1: Reg = Reg::new(6, true);
pub const REG_T2: Reg = Reg::new(7, true);
pub const REG_S0: Reg = Reg::new(8, true); //栈帧寄存器
pub const REG_S1: Reg = Reg::new(9, true); //保留寄存器
pub const REG_A0: Reg = Reg::new(10, true); //返回值寄存器 以及 函数参数寄存器
pub const REG_A1: Reg = Reg::new(11, true);
pub const REG_A2: Reg = Reg::new(12, true);
pub const REG_A3: Reg = Reg::new(13, true);
pub const REG_A4: Reg = Reg::new(14, true);
pub const REG_A5: Reg = Reg::new(15, true);
pub const REG_A6: Reg = Reg::new(16, true);
pub const REG_A7: Reg = Reg::new(17, true);
pub const REG_S2: Reg = Reg::new(18, true);
pub const REG_S3: Reg = Reg::new(19, true);
pub const REG_S4: Reg = Reg::new(20, true);
pub const REG_S5: Reg = Reg::new(21, true);
pub const REG_S6: Reg = Reg::new(22, true);
pub const REG_S7: Reg = Reg::new(23, true);
pub const REG_S8: Reg = Reg::new(24, true);
pub const REG_S9: Reg = Reg::new(25, true);
pub const REG_S10: Reg = Reg::new(26, true);
pub const REG_S11: Reg = Reg::new(27, true);
pub const REG_T3: Reg = Reg::new(28, true);
pub const REG_T4: Reg = Reg::new(29, true);
pub const REG_T5: Reg = Reg::new(30, true);
pub const REG_T6: Reg = Reg::new(31, true);

// 浮点寄存器
pub const REG_FT0: Reg = Reg::new(0, false);
pub const REG_FT1: Reg = Reg::new(1, false);
pub const REG_FT2: Reg = Reg::new(2, false);
pub const REG_FT3: Reg = Reg::new(3, false);
pub const REG_FT4: Reg = Reg::new(4, false);
pub const REG_FT5: Reg = Reg::new(5, false);
pub const REG_FT6: Reg = Reg::new(6, false);
pub const REG_FT7: Reg = Reg::new(7, false);
pub const REG_FS0: Reg = Reg::new(8, false);
pub const REG_FS1: Reg = Reg::new(9, false);
pub const REG_FA0: Reg = Reg::new(10, false);
pub const REG_FA1: Reg = Reg::new(11, false);
pub const REG_FA2: Reg = Reg::new(12, false);
pub const REG_FA3: Reg = Reg::new(13, false);
pub const REG_FA4: Reg = Reg::new(14, false);
pub const REG_FA5: Reg = Reg::new(15, false);
pub const REG_FA6: Reg = Reg::new(16, false);
pub const REG_FA7: Reg = Reg::new(17, false);
pub const REG_FS2: Reg = Reg::new(18, false);
pub const REG_FS3: Reg = Reg::new(19, false);
pub const REG_FS4: Reg = Reg::new(20, false);
pub const REG_FS5: Reg = Reg::new(21, false);
pub const REG_FS6: Reg = Reg::new(22, false);
pub const REG_FS7: Reg = Reg::new(23, false);
pub const REG_FS8: Reg = Reg::new(24, false);
pub const REG_FS9: Reg = Reg::new(25, false);
pub const REG_FS10: Reg = Reg::new(26, false);
pub const REG_FS11: Reg = Reg::new(27, false);
pub const REG_FT8: Reg = Reg::new(28, false);
pub const REG_FT9: Reg = Reg::new(29, false);
pub const REG_FT10: Reg = Reg::new(30, false);
pub const REG_FT11: Reg = Reg::new(31, false);

static USUAL_REG_COUNTER: Lazy<ParalCounter> = Lazy::new(|| ParalCounter::new(32, 100_000_000));
static FLOAT_REG_COUNTER: Lazy<ParalCounter> = Lazy::new(|| ParalCounter::new(32, 100_000_000));
impl Reg {
    #[inline]
    pub fn gen_asm(&self) -> String {
        if self.is_phisic() {
            match self.is_usual {
                true => match self.id {
                    0 => String::from("zero"),
                    1 => String::from("ra"),
                    2 => String::from("sp"),
                    3 => String::from("gp"),
                    4 => String::from("tp"),
                    5 => String::from("t0"),
                    6 => String::from("t1"),
                    7 => String::from("t2"),
                    8 => String::from("s0"),
                    9 => String::from("s1"),
                    10 => String::from("a0"),
                    11 => String::from("a1"),
                    12 => String::from("a2"),
                    13 => String::from("a3"),
                    14 => String::from("a4"),
                    15 => String::from("a5"),
                    16 => String::from("a6"),
                    17 => String::from("a7"),
                    18 => String::from("s2"),
                    19 => String::from("s3"),
                    20 => String::from("s4"),
                    21 => String::from("s5"),
                    22 => String::from("s6"),
                    23 => String::from("s7"),
                    24 => String::from("s8"),
                    25 => String::from("s9"),
                    26 => String::from("s10"),
                    27 => String::from("s11"),
                    28 => String::from("t3"),
                    29 => String::from("t4"),
                    30 => String::from("t5"),
                    31 => String::from("t6"),
                    _ => panic!("Invalid register id"),
                },
                false => match self.id {
                    0 => String::from("ft0"),
                    1 => String::from("ft1"),
                    2 => String::from("ft2"),
                    3 => String::from("ft3"),
                    4 => String::from("ft4"),
                    5 => String::from("ft5"),
                    6 => String::from("ft6"),
                    7 => String::from("ft7"),
                    8 => String::from("fs0"),
                    9 => String::from("fs1"),
                    10 => String::from("fa0"),
                    11 => String::from("fa1"),
                    12 => String::from("fa2"),
                    13 => String::from("fa3"),
                    14 => String::from("fa4"),
                    15 => String::from("fa5"),
                    16 => String::from("fa6"),
                    17 => String::from("fa7"),
                    18 => String::from("fs2"),
                    19 => String::from("fs3"),
                    20 => String::from("fs4"),
                    21 => String::from("fs5"),
                    22 => String::from("fs6"),
                    23 => String::from("fs7"),
                    24 => String::from("fs8"),
                    25 => String::from("fs9"),
                    26 => String::from("fs10"),
                    27 => String::from("fs11"),
                    28 => String::from("ft8"),
                    29 => String::from("ft9"),
                    30 => String::from("ft10"),
                    31 => String::from("ft11"),
                    _ => panic!("Invalid register id"),
                },
            }
        } else {
            panic!("gen_asm for virtual reg is not implemented");
        }
    }

    #[inline]
    pub fn to_str(&self) -> String {
        match self.is_usual {
            true => format!("x{}", self.id),
            false => format!("f{}", self.id),
        }
    }

    #[inline]
    pub fn is_phisic(&self) -> bool {
        (0..=31).contains(&self.id)
    }
    #[inline]
    pub fn is_virtual(&self) -> bool {
        !self.is_phisic()
    }
    // 生成一个虚拟通用寄存器
    pub fn gen_virtual_usual_reg() -> Self {
        let id = USUAL_REG_COUNTER.get_id().unwrap();
        Self::new(id as u32, true)
    }
    // 生成一个虚拟浮点寄存器
    pub fn gen_virtual_float_reg() -> Self {
        let id = FLOAT_REG_COUNTER.get_id().unwrap();
        Self::new(id as u32, false)
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
        self.0.clone()
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

#[derive(Clone)]
pub enum Inst {
    // 运算类型指令
    Add(AddInst),
    Mul(MulInst),
    Div(DivInst),
    SLL(SllInst), //逻辑左移
    SRL(SrlInst), //逻辑右移
    Neg(NegInst),
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

#[derive(Clone)]
pub struct AddInst(Operand, Operand, Operand);
#[derive(Clone)]
pub struct MulInst(Operand, Operand, Operand);
#[derive(Clone)]
pub struct DivInst(Operand, Operand, Operand);

#[derive(Clone)]
pub struct SllInst(Operand, Operand, Operand);
#[derive(Clone)]
pub struct SrlInst(Operand, Operand, Operand);
#[derive(Clone)]
pub struct NegInst(Operand, Operand);

#[derive(Clone)]
pub struct MvInst(Operand, Operand);
#[derive(Clone)]
pub struct LdInst(Operand, Operand, Operand);
#[derive(Clone)]
// to_store , offset , base
pub struct SdInst(Operand, Operand, Operand);
#[derive(Clone)]
pub struct LaInst(Operand, Operand);
#[derive(Clone)]
pub struct JmpInst(Operand);
#[derive(Clone)]
pub struct BranchInst(BranchOp, Operand, Operand, Operand);
#[derive(Clone)]
pub struct CallInst(Operand);
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

impl AddInst {
    pub fn gen_asm(&self) -> String {
        let dst = self.0.gen_asm();
        let lhs = self.1.gen_asm();
        let rhs = self.2.gen_asm();
        format!("add {},{},{}", dst, lhs, rhs)
    }
}
impl MulInst {
    pub fn gen_asm(&self) -> String {
        let dst = self.0.gen_asm();
        let lhs = self.1.gen_asm();
        let rhs = self.2.gen_asm();
        format!("mul {},{},{}", dst, lhs, rhs)
    }
    pub fn optimize(&self) -> Vec<Inst> {
        //TODO, 判断是否有优化必要
        // 把惩罚指令判断是否能够优化成移位指令
        // 如果乘法指令中的一个是常数,且该常数能够拆分成2的幂次方,则可以优化成移位指令
        let mut ret: Vec<Inst> = vec![];
        // 如果第二个数不是常数,没有优化必要
        if self.2.imm().is_none() {
            ret.push(Inst::Mul(self.clone()));
            return ret;
        }
        let imm = self.2.imm().unwrap().0;
        match imm.cmp(&0) {
            std::cmp::Ordering::Less => {
                ret.push(Inst::Mul(self.clone()));
                todo!("优化乘以负常数的情况")
            }
            std::cmp::Ordering::Equal => {
                // 优化乘以0的情况,则把zero赋予,事实上,不应该出现乘0指令,如果出现,中应该能够优化掉
                ret.push(Inst::Mv(MvInst(self.0.clone(), Operand::Reg(REG_ZERO))));
            }
            std::cmp::Ordering::Greater => {
                todo!("优化乘以正整数的情况")
            }
        }
        ret
    }
}
impl DivInst {
    pub fn gen_asm(&self) -> String {
        let dst = self.0.gen_asm();
        let lhs = self.1.gen_asm();
        let rhs = self.2.gen_asm();
        format!("div {},{},{}", dst, lhs, rhs)
    }
    pub fn optimize(&self) -> Vec<Inst> {
        //TODO, 判断是否有优化必要
        vec![Inst::Div(self.to_owned())]
    }
}
impl SllInst {
    pub fn gen_asm(&self) -> String {
        let dst = self.0.gen_asm();
        let lhs = self.1.gen_asm();
        let rhs = self.2.gen_asm();
        format!("sll {},{},{}", dst, lhs, rhs)
    }
}
impl SrlInst {
    pub fn gen_asm(&self) -> String {
        let dst = self.0.gen_asm();
        let lhs = self.1.gen_asm();
        let rhs = self.2.gen_asm();
        format!("srl {},{},{}", dst, lhs, rhs)
    }
}
impl NegInst {
    pub fn gen_asm(&self) -> String {
        let dst = self.0.gen_asm();
        let src = self.1.gen_asm();
        format!("neg {},{}", dst, src)
    }
}
impl MvInst {
    pub fn gen_asm(&self) -> String {
        format!("mv {},{}", self.0.gen_asm(), self.1.gen_asm())
    }
}
impl LdInst {
    pub fn gen_asm(&self) -> String {
        let to_store = self.0.gen_asm();
        let offset = self.1.gen_asm();
        let base = self.2.gen_asm();
        format!("ld {},{}({})", to_store, offset, base)
    }
}
impl SdInst {
    pub fn gen_asm(&self) -> String {
        let to_store = self.0.gen_asm();
        let offset = self.1.gen_asm();
        let base = self.2.gen_asm();
        format!("sd {},{}({})", to_store, offset, base)
    }
}
impl LaInst {
    pub fn gen_asm(&self) -> String {
        format!("la {},{}", self.0.gen_asm(), self.1.gen_asm())
    }
}
impl JmpInst {
    pub fn gen_asm(&self) -> String {
        format!("j {}", self.0.gen_asm())
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
impl CallInst {
    pub fn gen_asm(&self) -> String {
        format!("call {}", self.0.gen_asm())
    }
}

impl Inst {
    pub fn gen_asm(&self) -> String {
        match self {
            Inst::Add(inst) => inst.gen_asm(),
            Inst::Mul(inst) => inst.gen_asm(),
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

/// 单元测试
#[cfg(test)]
pub mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_gen_reg() {
        let mut regs = HashSet::new();
        let mut handlers = vec![];
        for _ in 0..10 {
            let handler = std::thread::spawn(move || {
                let mut regs = HashSet::new();
                for _ in 0..1000 {
                    let reg = Reg::gen_virtual_usual_reg();
                    regs.insert(reg.clone());
                    let reg = Reg::gen_virtual_float_reg();
                    regs.insert(reg.clone());
                }
                regs
            });
            handlers.push(handler);
        }
        for handler in handlers {
            let par_regs = handler.join().unwrap();
            regs.extend(par_regs.iter().cloned())
        }
        assert_eq!(regs.len(), 20000);
        for reg in regs.iter() {
            assert!(reg.is_virtual());
        }
        for i in 0..10000 {
            let reg = Reg::new(i + 32, true);
            assert!(regs.contains(&reg));
            let reg = Reg::new(i + 32, false);
            assert!(regs.contains(&reg));
        }
    }

    #[test]
    fn test_special_reg() {
        let reg = Reg::new(0, true);
        assert_eq!(reg.gen_asm(), "zero");
        let reg = Reg::new(1, true);
        assert_eq!(reg.gen_asm(), "ra");
        let reg = Reg::new(2, true);
        assert_eq!(reg.gen_asm(), "sp");
        let reg = Reg::new(3, true);
        assert_eq!(reg.gen_asm(), "gp");
        let reg = Reg::new(4, true);
        assert_eq!(reg.gen_asm(), "tp");
    }
    #[test]
    fn test_usual_float() {
        for i in 0..=31 {
            let reg = Reg::new(i, true);
            assert_eq!(reg.to_str(), format!("x{}", i));
            assert!(reg.is_usual());
            let reg = Reg::new(i, false);
            assert_eq!(reg.to_str(), format!("f{}", i));
            assert!(!reg.is_usual());
        }
    }
    #[test]
    fn test_phisic_virtual() {
        for i in 0..=31 {
            let reg = Reg::new(i, true);
            assert!(reg.is_phisic());
            assert!(!reg.is_virtual());
            let reg = Reg::new(i, false);
            assert!(reg.is_phisic());
            assert!(!reg.is_virtual());
        }
        for i in 32..=127 {
            let reg = Reg::new(i, false);
            assert!(!reg.is_phisic());
            assert!(reg.is_virtual());
        }
    }
    #[test]
    pub fn test_const_phisic_reg() {
        assert_eq!(REG_ZERO.gen_asm(), "zero");
        assert_eq!(REG_RA.gen_asm(), "ra");
        assert_eq!(REG_SP.gen_asm(), "sp");
        assert_eq!(REG_GP.gen_asm(), "gp");
        assert_eq!(REG_TP.gen_asm(), "tp");
        assert_eq!(REG_T0.gen_asm(), "t0");
        assert_eq!(REG_T1.gen_asm(), "t1");
        assert_eq!(REG_T2.gen_asm(), "t2");
        assert_eq!(REG_S0.gen_asm(), "s0");
        assert_eq!(REG_S1.gen_asm(), "s1");
        assert_eq!(REG_A0.gen_asm(), "a0");
        assert_eq!(REG_A1.gen_asm(), "a1");
        assert_eq!(REG_A2.gen_asm(), "a2");
        assert_eq!(REG_A3.gen_asm(), "a3");
        assert_eq!(REG_A4.gen_asm(), "a4");
        assert_eq!(REG_A5.gen_asm(), "a5");
        assert_eq!(REG_A6.gen_asm(), "a6");
        assert_eq!(REG_A7.gen_asm(), "a7");
        assert_eq!(REG_S2.gen_asm(), "s2");
        assert_eq!(REG_S3.gen_asm(), "s3");
        assert_eq!(REG_S4.gen_asm(), "s4");
        assert_eq!(REG_S5.gen_asm(), "s5");
        assert_eq!(REG_S6.gen_asm(), "s6");
        assert_eq!(REG_S7.gen_asm(), "s7");
        assert_eq!(REG_S8.gen_asm(), "s8");
        assert_eq!(REG_S9.gen_asm(), "s9");
        assert_eq!(REG_S10.gen_asm(), "s10");
        assert_eq!(REG_S11.gen_asm(), "s11");
        assert_eq!(REG_T3.gen_asm(), "t3");
        assert_eq!(REG_T4.gen_asm(), "t4");
        assert_eq!(REG_T5.gen_asm(), "t5");
        assert_eq!(REG_T6.gen_asm(), "t6");
        assert_eq!(REG_FT0.gen_asm(), "ft0");
        assert_eq!(REG_FT1.gen_asm(), "ft1");
        assert_eq!(REG_FT2.gen_asm(), "ft2");
        assert_eq!(REG_FT3.gen_asm(), "ft3");
        assert_eq!(REG_FT4.gen_asm(), "ft4");
        assert_eq!(REG_FT5.gen_asm(), "ft5");
        assert_eq!(REG_FT6.gen_asm(), "ft6");
        assert_eq!(REG_FT7.gen_asm(), "ft7");
        assert_eq!(REG_FS0.gen_asm(), "fs0");
        assert_eq!(REG_FS1.gen_asm(), "fs1");
        assert_eq!(REG_FA0.gen_asm(), "fa0");
        assert_eq!(REG_FA1.gen_asm(), "fa1");
        assert_eq!(REG_FA2.gen_asm(), "fa2");
        assert_eq!(REG_FA3.gen_asm(), "fa3");
        assert_eq!(REG_FA4.gen_asm(), "fa4");
        assert_eq!(REG_FA5.gen_asm(), "fa5");
        assert_eq!(REG_FA6.gen_asm(), "fa6");
        assert_eq!(REG_FA7.gen_asm(), "fa7");
        assert_eq!(REG_FS2.gen_asm(), "fs2");
        assert_eq!(REG_FS3.gen_asm(), "fs3");
        assert_eq!(REG_FS4.gen_asm(), "fs4");
        assert_eq!(REG_FS5.gen_asm(), "fs5");
        assert_eq!(REG_FS6.gen_asm(), "fs6");
        assert_eq!(REG_FS7.gen_asm(), "fs7");
        assert_eq!(REG_FS8.gen_asm(), "fs8");
        assert_eq!(REG_FS9.gen_asm(), "fs9");
        assert_eq!(REG_FS10.gen_asm(), "fs10");
        assert_eq!(REG_FS11.gen_asm(), "fs11");
        assert_eq!(REG_FT8.gen_asm(), "ft8");
        assert_eq!(REG_FT9.gen_asm(), "ft9");
        assert_eq!(REG_FT10.gen_asm(), "ft10");
        assert_eq!(REG_FT11.gen_asm(), "ft11");
    }
}
