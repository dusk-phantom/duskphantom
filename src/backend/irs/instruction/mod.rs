mod algebra;
mod data_move;
mod inst;
mod reg_def_use;
mod control_flow;
pub use super::*;
pub use algebra::*;
pub use data_move::*;
pub use control_flow::*;
pub use inst::*;
pub use reg_def_use::*;
pub use crate::{impl_inst_from, impl_mem_inst, impl_three_op_inst, impl_two_op_inst, impl_unary_inst};

// impl From<T> for Inst

// for algebraic operation
impl_inst_from!(AddInst, Add);
impl_inst_from!(SubInst, Sub);
impl_inst_from!(MulInst, Mul);
impl_inst_from!(RemInst, Rem);
impl_inst_from!(DivInst, Div);
impl_inst_from!(NegInst, Neg);

// for bit count operation
impl_inst_from!(AndInst,And);
impl_inst_from!(OrInst,Or);
impl_inst_from!(XorInst,Xor);
impl_inst_from!(SllInst, Sll);
impl_inst_from!(SrlInst, Srl);
impl_inst_from!(SltInst,Slt);

// inst for data transfer
impl_inst_from!(MvInst, Mv);
impl_inst_from!(LaInst, La);
impl_inst_from!(SdInst, Sd);
impl_inst_from!(LdInst, Ld);
impl_inst_from!(LwInst,Lw);
impl_inst_from!(SwInst,Sw);

// inst for control flow
impl_inst_from!(JmpInst, Jmp);
impl_inst_from!(BranchInst, Branch);
impl_inst_from!(CallInst, Call);
impl_inst_from!(TailInst, Tail);
