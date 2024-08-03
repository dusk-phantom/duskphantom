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
            Inst::Lla(inst) => inst.uses(),
            Inst::Jmp(inst) => inst.uses(),
            Inst::Beq(inst) => inst.uses(),
            Inst::Bne(inst) => inst.uses(),
            Inst::Bge(inst) => inst.uses(),
            Inst::Blt(inst) => inst.uses(),
            Inst::Bgt(inst) => inst.uses(),
            Inst::Ble(inst) => inst.uses(),
            Inst::Call(inst) => inst.uses(),
            Inst::SRA(inst) => inst.uses(),
            Inst::And(inst) => inst.uses(),
            Inst::Or(inst) => inst.uses(),
            Inst::Xor(inst) => inst.uses(),
            Inst::Ret => vec![],
            Inst::Tail(tail) => tail.uses(),
            Inst::Li(inst) => inst.uses(),
            Inst::Seqz(inst) => inst.uses(),
            Inst::I2f(i2f) => i2f.uses(),
            Inst::F2i(f2i) => f2i.uses(),
            Inst::Snez(snez) => snez.uses(),
            Inst::Not(not) => not.uses(),
            Inst::LocalAddr(laddr) => laddr.uses(),
            Inst::Sltu(sltu) => sltu.uses(),
            Inst::Sgtu(sgtu) => sgtu.uses(),
            Inst::UDiv(udiv) => udiv.uses(),
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
            Inst::Xor(inst) => inst.defs(),
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
            Inst::Sw(inst) => inst.defs(),
            Inst::Lla(inst) => inst.defs(),
            Inst::Jmp(inst) => inst.defs(),
            Inst::Beq(inst) => inst.defs(),
            Inst::Bne(inst) => inst.defs(),
            Inst::Bge(inst) => inst.defs(),
            Inst::Blt(inst) => inst.defs(),
            Inst::Bgt(inst) => inst.defs(),
            Inst::Ble(inst) => inst.defs(),
            Inst::Call(inst) => inst.defs(),
            Inst::Ret => vec![],
            Inst::Tail(tail) => tail.defs(),
            Inst::Slt(inst) => inst.defs(),
            Inst::Li(inst) => inst.defs(),
            Inst::Seqz(inst) => inst.defs(),
            Inst::I2f(i2f) => i2f.defs(),
            Inst::F2i(f2i) => f2i.defs(),
            Inst::Snez(snez) => snez.defs(),
            Inst::Not(not) => not.defs(),
            Inst::LocalAddr(laddr) => laddr.defs(),
            Inst::Sltu(sltu) => sltu.defs(),
            Inst::Sgtu(sgtu) => sgtu.defs(),
            Inst::UDiv(udiv) => udiv.defs(),
        }
    }
}

impl RegUses for LlaInst {}

impl RegDefs for LlaInst {
    fn defs(&self) -> Vec<&Reg> {
        vec![self.dst()]
    }
}

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

impl RegDefs for LoadInst {
    fn defs(&self) -> Vec<&Reg> {
        vec![self.dst()]
    }
}
impl RegUses for LoadInst {}
impl RegDefs for StoreInst {}
impl RegUses for StoreInst {
    fn uses(&self) -> Vec<&Reg> {
        vec![self.src()]
    }
}
impl RegDefs for TailInst {}
impl RegUses for TailInst {}

impl RegDefs for LocalAddr {
    fn defs(&self) -> Vec<&Reg> {
        vec![self.dst()]
    }
}
impl RegUses for LocalAddr {}
