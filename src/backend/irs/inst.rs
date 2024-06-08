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
#[derive(Clone)]
pub struct AddInst(Operand, Operand, Operand);

#[derive(Clone)]
pub struct SubInst(Operand, Operand, Operand);

#[derive(Clone)]
pub struct MulInst(Operand, Operand, Operand);

// rem
#[derive(Clone)]
pub struct RemInst(Operand, Operand, Operand);

#[derive(Clone)]
pub struct DivInst(Operand, Operand, Operand);

// slli
// slliw
// sllw
#[derive(Clone)]
pub struct SllInst(Operand, Operand, Operand);

// sra 算术右移
// srai
// sraiw
// sraw
// srli
// srliw
// srlw
#[derive(Clone)]
pub struct SrlInst(Operand, Operand, Operand);

// ori
// or
// not
// negw
#[derive(Clone)]
pub struct NegInst(Operand, Operand);

#[derive(Clone)]
pub struct MvInst(Operand, Operand);

// lw
#[derive(Clone)]
pub struct LdInst(Operand, Operand, Operand);

/// (to_store , offset , base)
// sw
#[derive(Clone)]
pub struct SdInst(Operand, Operand, Operand);

// lla
#[derive(Clone)]
pub struct LaInst(Operand, Operand);

// j
// jal
// jalr
// jr
#[derive(Clone)]
pub struct JmpInst(Operand);

#[derive(Clone)]
pub struct BranchInst(BranchOp, Operand, Operand, Operand);
#[derive(Clone)]
pub struct CallInst(Operand);

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

impl SubInst {
    pub fn gen_asm(&self) -> String {
        let dst = self.0.gen_asm();
        let lhs = self.1.gen_asm();
        let rhs = self.2.gen_asm();
        format!("sub {},{},{}", dst, lhs, rhs)
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
        let imm = &self.2.imm().unwrap();
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

impl RemInst {
    pub fn gen_asm(&self) -> String {
        let dst = self.0.gen_asm();
        let lhs = self.1.gen_asm();
        let rhs = self.2.gen_asm();
        format!("rem {},{},{}", dst, lhs, rhs)
    }
}

impl DivInst {
    pub fn gen_asm(&self) -> String {
        let dst = self.0.gen_asm();
        let lhs = self.1.gen_asm();
        let rhs = self.2.gen_asm();
        format!("div {},{},{}", dst, lhs, rhs)
    }
    pub fn optimize(&self) -> Option<Vec<Inst>> {
        todo!("判断是否有优化必要,如果有,返回优化产生的指令");
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
