use anyhow::Ok;

use super::*;

// 实现一些用于辅助的伪指令
#[derive(Clone, Debug)]
pub struct StoreInst {
    dst: StackSlot,
    src: Reg,
}
#[derive(Clone, Debug)]
pub struct LoadInst {
    dst: Reg,
    src: StackSlot,
}
impl StoreInst {
    pub fn new(dst: StackSlot, src: Reg) -> Self {
        Self { dst, src }
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
}

impl LoadInst {
    pub fn new(dst: Reg, src: StackSlot) -> Self {
        Self { dst, src }
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
}

impl_mem_inst!(LdInst, "ld");
impl_mem_inst!(SdInst, "sd");
impl_mem_inst!(SwInst, "sw");
impl_mem_inst!(LwInst, "lw");
impl_two_op_inst!(LiInst, "li");

// la
#[derive(Clone, Debug)]
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
    pub fn gen_asm(&self) -> String {
        format!("la {},{}", self.0.gen_asm(), self.1.gen_asm())
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
        if self.base() == &from {
            *self.base_mut() = to;
        }
        Ok(())
    }
}
impl RegReplace for SwInst {
    fn replace_use(&mut self, from: Reg, to: Reg) -> Result<()> {
        if self.base() == &from {
            *self.base_mut() = to;
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

impl RegReplace for LaInst {
    fn replace_def(&mut self, from: Reg, to: Reg) -> Result<()> {
        if self.0 == from {
            self.0 = to;
        }
        Ok(())
    }
}
