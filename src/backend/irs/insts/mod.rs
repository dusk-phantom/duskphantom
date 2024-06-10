mod three_op;
mod binary;
mod mem;
mod unary;
mod branch;
mod inst;
mod reg_def_use;
pub use super::*;
pub use three_op::*;
pub use binary::*;
pub use mem::*;
pub use unary::*;
pub use branch::*;
pub use inst::*;
pub use reg_def_use::*;
pub use crate::{impl_inst_from, impl_mem_inst, impl_three_op_inst, impl_two_op_inst, impl_unary_inst};

// impl From<T> for Inst
impl_inst_from!(SdInst, Sd);
impl_inst_from!(LdInst, Ld);
impl_inst_from!(AddInst, Add);
impl_inst_from!(SubInst, Sub);
impl_inst_from!(MulInst, Mul);
impl_inst_from!(RemInst, Rem);
impl_inst_from!(DivInst, Div);
impl_inst_from!(SllInst, SLL);
impl_inst_from!(SrlInst, SRL);
impl_inst_from!(NegInst, Neg);
impl_inst_from!(MvInst, Mv);
impl_inst_from!(JmpInst, Jmp);
impl_inst_from!(BranchInst, Branch);
impl_inst_from!(CallInst, Call);
impl_inst_from!(LaInst, La);