use super::*;

#[derive(Debug, Clone)]
pub struct CallInst(Label);
impl CallInst {
    pub fn new(dst: Label) -> Self {
        Self(dst)
    }
    pub fn func_name(&self) -> &Label {
        &self.0
    }
    pub fn gen_asm(&self) -> String {
        let dst = self.func_name().gen_asm();
        format!("call {}", dst)
    }
}

impl_unary_inst!(JmpInst, "j");
// impl_unary_inst!(CallInst, "call");
impl_unary_inst!(TailInst, "tail");

impl_branch_inst!(BeqInst, "beq");
impl_branch_inst!(BneInst, "bne");
impl_branch_inst!(BltInst, "blt");
impl_branch_inst!(BleInst, "ble");
impl_branch_inst!(BgtInst, "bgt");
impl_branch_inst!(BgeInst, "bge");
