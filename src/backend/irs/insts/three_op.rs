use super::*;

impl_three_op_inst!(AddInst, "add");
impl_three_op_inst!(SubInst, "sub");
impl_three_op_inst!(MulInst, "mul");
impl_three_op_inst!(RemInst, "rem");
impl_three_op_inst!(DivInst, "div");
impl_three_op_inst!(SllInst, "sll");
impl_three_op_inst!(SrlInst, "srl");
impl_three_op_inst!(SraInst, "sra");