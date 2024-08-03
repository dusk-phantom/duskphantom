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

#[derive(Clone, Debug)]
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
    Not(NotInst),
    And(AndInst),
    Or(OrInst),
    Xor(XorInst),
    Neg(NegInst),

    // comparison operation
    Slt(SltInst),
    Sltu(SltuInst),
    Sgtu(SgtuInst),
    Seqz(SeqzInst),
    Snez(SnezInst),

    // data transfer operation
    Mv(MvInst),
    Li(LiInst),
    Ld(LdInst),
    Sd(SdInst),
    Lw(LwInst),
    Sw(SwInst),
    Lla(LlaInst),
    // special load and store
    Load(LoadInst),
    Store(StoreInst),
    // special load address of local var
    LocalAddr(LocalAddr),

    // conversion operation
    I2f(I2fInst),
    F2i(F2iInst),

    // control flow operation
    Jmp(JmpInst),

    Beq(BeqInst),
    Bne(BneInst),
    Blt(BltInst),
    Ble(BleInst),
    Bgt(BgtInst),
    Bge(BgeInst),

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
            Inst::Lla(inst) => inst.gen_asm(),
            Inst::Li(inst) => inst.gen_asm(),
            Inst::Load(inst) => inst.gen_asm(),
            Inst::Store(inst) => inst.gen_asm(),
            Inst::Jmp(inst) => inst.gen_asm(),
            Inst::Beq(inst) => inst.gen_asm(),
            Inst::Bne(inst) => inst.gen_asm(),
            Inst::Bge(inst) => inst.gen_asm(),
            Inst::Blt(inst) => inst.gen_asm(),
            Inst::Bgt(inst) => inst.gen_asm(),
            Inst::Ble(inst) => inst.gen_asm(),
            Inst::Call(inst) => inst.gen_asm(),
            Inst::SRA(inst) => inst.gen_asm(),
            Inst::Ret => String::from("ret"),
            Inst::And(inst) => inst.gen_asm(),
            Inst::Or(inst) => inst.gen_asm(),
            Inst::Xor(inst) => inst.gen_asm(),
            Inst::Tail(inst) => inst.gen_asm(),
            Inst::Seqz(inst) => inst.gen_asm(),
            Inst::I2f(i2f) => i2f.gen_asm(),
            Inst::F2i(f2i) => f2i.gen_asm(),
            Inst::Snez(snez) => snez.gen_asm(),
            Inst::Not(not) => not.gen_asm(),
            Inst::LocalAddr(local_addr) => local_addr.gen_asm(),
            Inst::Sltu(sltu) => sltu.gen_asm(),
            Inst::Sgtu(sgtu) => sgtu.gen_asm(),
        }
    }
}

//*********************************************************************************
// impl RegReplace for Inst
// replace the use and def register of the instruction
// which is used in the register allocation phase
//*********************************************************************************
pub trait RegReplace {
    #[allow(unused_variables)]
    fn replace_use(&mut self, from: Reg, to: Reg) -> Result<()> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn replace_def(&mut self, from: Reg, to: Reg) -> Result<()> {
        Ok(())
    }
}
impl RegReplace for Inst {
    fn replace_use(&mut self, from: Reg, to: Reg) -> Result<()> {
        match self {
            Inst::Add(inst) => inst.replace_use(from, to),
            Inst::Sub(inst) => inst.replace_use(from, to),
            Inst::Mul(inst) => inst.replace_use(from, to),
            Inst::Rem(inst) => inst.replace_use(from, to),
            Inst::Div(inst) => inst.replace_use(from, to),
            Inst::Sll(inst) => inst.replace_use(from, to),
            Inst::Srl(inst) => inst.replace_use(from, to),
            Inst::Neg(inst) => inst.replace_use(from, to),
            Inst::Slt(inst) => inst.replace_use(from, to),
            Inst::Mv(inst) => inst.replace_use(from, to),
            Inst::Ld(inst) => inst.replace_use(from, to),
            Inst::Sd(inst) => inst.replace_use(from, to),
            Inst::Lw(inst) => inst.replace_use(from, to),
            Inst::Sw(inst) => inst.replace_use(from, to),
            Inst::Load(inst) => inst.replace_use(from, to),
            Inst::Store(inst) => inst.replace_use(from, to),
            Inst::Lla(inst) => inst.replace_use(from, to),
            Inst::Li(inst) => inst.replace_use(from, to),
            Inst::I2f(i2f) => i2f.replace_use(from, to),
            Inst::F2i(f2i) => f2i.replace_use(from, to),
            Inst::Jmp(inst) => inst.replace_use(from, to),
            Inst::Beq(inst) => inst.replace_use(from, to),
            Inst::Bne(inst) => inst.replace_use(from, to),
            Inst::Bge(inst) => inst.replace_use(from, to),
            Inst::Blt(inst) => inst.replace_use(from, to),
            Inst::Bgt(inst) => inst.replace_use(from, to),
            Inst::Ble(inst) => inst.replace_use(from, to),
            Inst::Call(inst) => inst.replace_use(from, to),
            Inst::SRA(inst) => inst.replace_use(from, to),
            Inst::Ret => Ok(()),
            Inst::And(inst) => inst.replace_use(from, to),
            Inst::Or(inst) => inst.replace_use(from, to),
            Inst::Xor(inst) => inst.replace_use(from, to),
            Inst::Tail(inst) => inst.replace_use(from, to),
            Inst::Seqz(inst) => inst.replace_use(from, to),
            Inst::Snez(snez) => snez.replace_use(from, to),
            Inst::Not(not) => not.replace_use(from, to),
            Inst::LocalAddr(laddr) => laddr.replace_use(from, to),
            Inst::Sltu(sltu) => sltu.replace_use(from, to),
            Inst::Sgtu(sgtu) => sgtu.replace_use(from, to),
        }
    }

    fn replace_def(&mut self, from: Reg, to: Reg) -> Result<()> {
        match self {
            Inst::Add(inst) => inst.replace_def(from, to),
            Inst::Sub(inst) => inst.replace_def(from, to),
            Inst::Mul(inst) => inst.replace_def(from, to),
            Inst::Rem(inst) => inst.replace_def(from, to),
            Inst::Div(inst) => inst.replace_def(from, to),
            Inst::Sll(inst) => inst.replace_def(from, to),
            Inst::Srl(inst) => inst.replace_def(from, to),
            Inst::Neg(inst) => inst.replace_def(from, to),
            Inst::Slt(inst) => inst.replace_def(from, to),
            Inst::Mv(inst) => inst.replace_def(from, to),
            Inst::Ld(inst) => inst.replace_def(from, to),
            Inst::Sd(inst) => inst.replace_def(from, to),
            Inst::Lw(inst) => inst.replace_def(from, to),
            Inst::Sw(inst) => inst.replace_def(from, to),
            Inst::Load(inst) => inst.replace_def(from, to),
            Inst::Store(inst) => inst.replace_def(from, to),
            Inst::Lla(inst) => inst.replace_def(from, to),
            Inst::Li(inst) => inst.replace_def(from, to),
            Inst::I2f(i2f) => i2f.replace_def(from, to),
            Inst::F2i(f2i) => f2i.replace_def(from, to),
            Inst::Jmp(inst) => inst.replace_def(from, to),
            Inst::Beq(inst) => inst.replace_def(from, to),
            Inst::Bne(inst) => inst.replace_def(from, to),
            Inst::Bge(inst) => inst.replace_def(from, to),
            Inst::Blt(inst) => inst.replace_def(from, to),
            Inst::Bgt(inst) => inst.replace_def(from, to),
            Inst::Ble(inst) => inst.replace_def(from, to),
            Inst::Call(inst) => inst.replace_def(from, to),
            Inst::SRA(inst) => inst.replace_def(from, to),
            Inst::Ret => Ok(()),
            Inst::And(inst) => inst.replace_def(from, to),
            Inst::Or(inst) => inst.replace_def(from, to),
            Inst::Xor(inst) => inst.replace_def(from, to),
            Inst::Tail(inst) => inst.replace_def(from, to),
            Inst::Seqz(inst) => inst.replace_def(from, to),
            Inst::Snez(snez) => snez.replace_def(from, to),
            Inst::Not(not) => not.replace_def(from, to),
            Inst::LocalAddr(laddr) => laddr.replace_def(from, to),
            Inst::Sltu(sltu) => sltu.replace_def(from, to),
            Inst::Sgtu(sgtu) => sgtu.replace_def(from, to),
        }
    }
}

//*********************************************************************************
// impl From<T> for Inst
// and impl TryFrom<Inst> for T
// T is the specific instruction type
//*********************************************************************************

// for algebraic operation
impl_inst_convert!(AddInst, Add);
impl_inst_convert!(SubInst, Sub);
impl_inst_convert!(MulInst, Mul);
impl_inst_convert!(RemInst, Rem);
impl_inst_convert!(DivInst, Div);
impl_inst_convert!(NegInst, Neg);

// for bit count operation
impl_inst_convert!(AndInst, And);
impl_inst_convert!(OrInst, Or);
impl_inst_convert!(XorInst, Xor);
impl_inst_convert!(SllInst, Sll);
impl_inst_convert!(SrlInst, Srl);
impl_inst_convert!(SraInst, SRA);
impl_inst_convert!(NotInst, Not);

// for comparison
impl_inst_convert!(SltInst, Slt);
impl_inst_convert!(SltuInst, Sltu);
impl_inst_convert!(SgtuInst, Sgtu);
impl_inst_convert!(SeqzInst, Seqz);
impl_inst_convert!(SnezInst, Snez);

// inst for data transfer
impl_inst_convert!(MvInst, Mv);
impl_inst_convert!(LlaInst, Lla);
impl_inst_convert!(SdInst, Sd);
impl_inst_convert!(LdInst, Ld);
impl_inst_convert!(LwInst, Lw);
impl_inst_convert!(SwInst, Sw);
impl_inst_convert!(LiInst, Li);
impl_inst_convert!(LoadInst, Load);
impl_inst_convert!(StoreInst, Store);
impl_inst_convert!(LocalAddr, LocalAddr);

// inst for conversion
impl_inst_convert!(I2fInst, I2f);
impl_inst_convert!(F2iInst, F2i);

// inst for control flow
impl_inst_convert!(JmpInst, Jmp);
impl_inst_convert!(CallInst, Call);
impl_inst_convert!(TailInst, Tail);
impl_inst_convert!(BeqInst, Beq);
impl_inst_convert!(BneInst, Bne);
impl_inst_convert!(BltInst, Blt);
impl_inst_convert!(BleInst, Ble);
impl_inst_convert!(BgtInst, Bgt);
impl_inst_convert!(BgeInst, Bge);
