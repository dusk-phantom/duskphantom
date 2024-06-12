
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
    Sub(SubInst),
    Mul(MulInst),
    Div(DivInst),
    Rem(RemInst),

    // bit count operation
    // xor
    // xori
    // subw
    Sll(SllInst),
    Srl(SrlInst),
    SRA(SraInst),
    And(AndInst),
    Or(OrInst),
    Xor(XorInst),
    Neg(NegInst),

    // comparison operation
    Slt(SltInst),

    // data transfer operation
    Mv(MvInst),
    Ld(LdInst),
    Sd(SdInst),
    Lw(LwInst),
    Sw(SwInst),
    La(LaInst),
    Load(LoadInst),
    Store(StoreInst),
    // control flow operation
    Jmp(JmpInst),
    Branch(BranchInst),
    Call(CallInst),
    Tail(TailInst),
    Ret,
}

// addi
// addiw
// addw
// and
// andi


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


// j
// jal
// jalr
// jr





impl Inst {
    pub fn gen_asm(&self) -> String {
        match self {
            Inst::Add(inst) => inst.gen_asm(),
            Inst::Sub(inst) => inst.gen_asm(),
            Inst::Mul(inst) => inst.gen_asm(),
            Inst::Rem(inst) => inst.gen_asm(),
            Inst::Neg(inst) => inst.gen_asm(),
            Inst::Div(inst) => inst.gen_asm(),
            Inst::Sll(inst) => inst.gen_asm(),
            Inst::Srl(inst) => inst.gen_asm(),
            Inst::Slt(inst) => inst.gen_asm(),
            Inst::Mv(inst) => inst.gen_asm(),
            Inst::Ld(inst) => inst.gen_asm(),
            Inst::Sd(inst) => inst.gen_asm(),
            Inst::Sw(inst) => inst.gen_asm(),
            Inst::Lw(inst) => inst.gen_asm(),
            Inst::La(inst) => inst.gen_asm(),
            Inst::Load(inst) => inst.gen_asm(),
            Inst::Store(inst) => inst.gen_asm(),
            Inst::Jmp(inst) => inst.gen_asm(),
            Inst::Branch(inst) => inst.gen_asm(),
            Inst::Call(inst) => inst.gen_asm(),
            Inst::SRA(inst) => inst.gen_asm(),
            Inst::Ret => String::from("ret"),
            Inst::And(inst) => inst.gen_asm(),
            Inst::Or(inst) => inst.gen_asm(),
            Inst::Xor(inst) => inst.gen_asm(),
            Inst::Tail(inst) =>inst.gen_asm(),
        }
    }
}




//* impl  get_stack_slot for mem Inst */
