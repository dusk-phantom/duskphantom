use super::*;

impl_unary_inst!(JmpInst, "j");
impl_unary_inst!(CallInst, "call");
impl_unary_inst!(TailInst,"tail");

impl_branch_inst!(BeqInst,"beq");
impl_branch_inst!(BneInst,"bne");
impl_branch_inst!(BltInst,"blt");
impl_branch_inst!(BleInst,"ble");
impl_branch_inst!(BgtInst,"bgt");
impl_branch_inst!(BgeInst,"bge");
