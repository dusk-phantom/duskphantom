use super::*;

impl_three_op_inst!(AddInst, "add");
impl_three_op_inst!(SubInst, "sub");
impl_three_op_inst!(MulInst, "mul");
impl_three_op_inst!(RemInst, "rem");
impl_three_op_inst!(DivInst, "div");
impl_three_op_inst!(SllInst, "sll");
impl_three_op_inst!(SrlInst, "srl");
impl_three_op_inst!(SraInst, "sra");
impl_three_op_inst!(AndInst, "and");
impl_two_op_inst!(NotInst, "not");
impl_three_op_inst!(OrInst, "or");
impl_three_op_inst!(XorInst, "xor");

// 实现比较指令
impl_three_op_inst!(SltInst, "slt");
impl_two_op_inst!(SnezInst, "snez");
impl_two_op_inst!(SeqzInst, "seqz");

impl_two_op_inst!(NegInst, "neg");
impl_two_op_inst!(MvInst, "mv");
