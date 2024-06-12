use super::*;

impl_mem_inst!(LdInst, "ld");
impl_mem_inst!(SdInst, "sd");
impl_mem_inst!(SwInst, "sw");
impl_mem_inst!(LwInst, "lw");


// la
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
    pub fn gen_asm(&self) -> String {
        format!("la {},{}", self.0.gen_asm(), self.1.gen_asm())
    }
}