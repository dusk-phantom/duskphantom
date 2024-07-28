use anyhow::Ok;

use super::*;

#[derive(Clone, Debug)]
pub enum MemSize {
    FourByte,
    EightByte,
}
impl MemSize {
    pub fn num_byte(&self) -> u32 {
        match self {
            MemSize::FourByte => 4,
            MemSize::EightByte => 8,
        }
    }
}

fn new_addr(ss: &StackSlot, stack_size: u32) -> (Imm, Reg) {
    if ss.start() > stack_size >> 1 {
        (ss.start().into(), REG_SP)
    } else {
        (((ss.start() as i64) - (stack_size as i64)).into(), REG_S0)
    }
}

// 实现一些用于辅助的伪指令
#[derive(Clone, Debug)]
pub struct StoreInst {
    dst: StackSlot,
    src: Reg,
    mode: MemSize,
}
#[derive(Clone, Debug)]
pub struct LoadInst {
    dst: Reg,
    src: StackSlot,
    mem_size: MemSize,
}
impl StoreInst {
    pub fn new(dst: StackSlot, src: Reg) -> Self {
        Self {
            dst,
            src,
            mode: MemSize::FourByte,
        }
    }

    pub fn with_8byte(mut self) -> Self {
        self.mode = MemSize::EightByte;
        self
    }
    pub fn dst(&self) -> &StackSlot {
        &self.dst
    }
    pub fn src(&self) -> &Reg {
        &self.src
    }
    pub fn dst_mut(&mut self) -> &mut StackSlot {
        &mut self.dst
    }

    pub fn src_mut(&mut self) -> &mut Reg {
        &mut self.src
    }
    pub fn gen_asm(&self) -> String {
        format!("store {},{}", self.src.gen_asm(), self.dst.gen_asm())
    }

    #[inline]
    pub fn phisicalize(&self, stack_size: u32) -> Result<Inst> {
        match self.mode {
            MemSize::FourByte => {
                let (off, base) = new_addr(&self.dst, stack_size);
                Ok(SwInst::new(self.src, off, base).into())
            }
            MemSize::EightByte => {
                let (off, base) = new_addr(&self.dst, stack_size);
                Ok(SdInst::new(self.src, off, base).into())
            }
        }
    }
}

impl LoadInst {
    /// new a load inst,default mem_size is four byte
    pub fn new(dst: Reg, src: StackSlot) -> Self {
        Self {
            dst,
            src,
            mem_size: MemSize::FourByte,
        }
    }
    pub fn with_8byte(mut self) -> Self {
        self.mem_size = MemSize::EightByte;
        self
    }

    #[inline]
    pub fn dst(&self) -> &Reg {
        &self.dst
    }
    #[inline]
    pub fn src(&self) -> &StackSlot {
        &self.src
    }
    #[inline]
    pub fn dst_mut(&mut self) -> &mut Reg {
        &mut self.dst
    }
    #[inline]
    pub fn src_mut(&mut self) -> &mut StackSlot {
        &mut self.src
    }
    #[inline]
    pub fn gen_asm(&self) -> String {
        format!("load {},{}", self.dst.gen_asm(), self.src.gen_asm())
    }

    #[inline]
    pub fn phisicalize(&self, stack_size: u32) -> Result<Inst> {
        match self.mem_size {
            MemSize::FourByte => {
                let (off, base) = new_addr(&self.src, stack_size);
                Ok(LwInst::new(self.dst, off, base).into())
            }
            MemSize::EightByte => {
                let (off, base) = new_addr(&self.src, stack_size);
                Ok(LdInst::new(self.dst, off, base).into())
            }
        }
    }
}

impl_mem_inst!(LdInst, "ld");
impl_mem_inst!(SdInst, "sd");
impl_mem_inst!(SwInst, "sw");
impl_mem_inst!(LwInst, "lw");
impl_two_op_inst!(LiInst, "li");

// la
#[derive(Clone, Debug)]
pub struct LlaInst(Reg, Label);
impl LlaInst {
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
    pub fn gen_asm(&self) -> String {
        format!("lla {},{}", self.0.gen_asm(), self.1.gen_asm())
    }
}

//*********************************************************************************
// impl RegReplace for data move inst
//*********************************************************************************

impl RegReplace for LoadInst {
    fn replace_def(&mut self, from: Reg, to: Reg) -> Result<()> {
        if self.dst() == &from {
            *self.dst_mut() = to;
        }
        Ok(())
    }
}
impl RegReplace for StoreInst {
    fn replace_use(&mut self, from: Reg, to: Reg) -> Result<()> {
        if self.src == from {
            self.src = to;
        }
        Ok(())
    }
}
impl RegReplace for LdInst {
    fn replace_def(&mut self, from: Reg, to: Reg) -> Result<()> {
        if self.dst() == &from {
            *self.dst_mut() = to;
        }
        Ok(())
    }
    fn replace_use(&mut self, from: Reg, to: Reg) -> Result<()> {
        if self.base() == &from {
            *self.base_mut() = to;
        }
        Ok(())
    }
}
impl RegReplace for SdInst {
    fn replace_use(&mut self, from: Reg, to: Reg) -> Result<()> {
        if self.base() == &from {
            *self.base_mut() = to;
        }
        if self.dst() == &from {
            *self.dst_mut() = to;
        }
        Ok(())
    }
}
impl RegReplace for SwInst {
    fn replace_use(&mut self, from: Reg, to: Reg) -> Result<()> {
        if self.base() == &from {
            *self.base_mut() = to;
        }
        if self.dst() == &from {
            *self.dst_mut() = to;
        }
        Ok(())
    }
}
impl RegReplace for LwInst {
    fn replace_def(&mut self, from: Reg, to: Reg) -> Result<()> {
        if self.dst() == &from {
            *self.dst_mut() = to;
        }
        Ok(())
    }
    fn replace_use(&mut self, from: Reg, to: Reg) -> Result<()> {
        if self.base() == &from {
            *self.base_mut() = to;
        }
        Ok(())
    }
}

impl RegReplace for LlaInst {
    fn replace_def(&mut self, from: Reg, to: Reg) -> Result<()> {
        if self.0 == from {
            self.0 = to;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gem_asm_mv() {
        let mv = MvInst::new(REG_A0.into(), REG_A1.into());
        assert_eq!(mv.gen_asm(), "mv a0,a1");
        let mv = MvInst::new(REG_FA0.into(), REG_FA1.into());
        assert_eq!(mv.gen_asm(), "fmv.s fa0,fa1");
    }
}
