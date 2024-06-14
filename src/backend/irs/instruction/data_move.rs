use super::*;

// 实现一些用于辅助的伪指令
#[derive(Clone)]
pub struct StoreInst{
    dst:Reg,
    src:StackSlot,
}
#[derive(Clone)]
pub struct LoadInst{
    dst:StackSlot,
    src:Reg,
}
impl StoreInst{
    pub fn new(dst:Reg,src:StackSlot)->Self{
        Self{
            dst,
            src,
        }
    }
    pub fn dst(&self)->&Reg{
        &self.dst
    }
    pub fn src(&self)->&StackSlot{
        &self.src
    }
    pub fn dst_mut(&mut self)->&mut Reg{
        &mut self.dst
    }
    pub fn src_mut(&mut self)->&mut StackSlot{
        &mut self.src
    }
    pub fn gen_asm(&self)->String{
        format!("store {},{}",self.dst.gen_asm(),self.src.start())
    }
}
impl LoadInst{
    pub fn new(dst:StackSlot,src:Reg)->Self{
        Self{
            dst,
            src,
        }
    }
    pub fn dst(&self)->&StackSlot{
        &self.dst
    
    }
    pub fn src(&self)->&Reg{
        &self.src
    }
    pub fn dst_mut(&mut self)->&mut StackSlot{
        &mut self.dst
    
    }
    pub fn src_mut(&mut self)->&mut Reg{
        &mut self.src
    }
    pub fn gen_asm(&self)->String{
        format!("load {},{}",self.dst.start(),self.src.gen_asm())
    }

}




impl_mem_inst!(LdInst, "ld");
impl_mem_inst!(SdInst, "sd");
impl_mem_inst!(SwInst, "sw");
impl_mem_inst!(LwInst, "lw");
impl_two_op_inst!(LiInst, "li");


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