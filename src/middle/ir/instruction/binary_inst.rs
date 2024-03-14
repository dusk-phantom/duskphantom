use super::*;
use crate::impl_binary_inst;

/// impl for binary operation and bitwise binary_inst
pub trait BinaryInst {
    fn get_lhs(&self) -> &Operand;
    fn set_lhs(&mut self, lhs: Operand);
    fn get_rhs(&self) -> &Operand;
    fn set_rhs(&mut self, rhs: Operand);
}

impl_binary_inst!(Add, "i32", get_add, lhs, rhs);
impl_binary_inst!(FAdd, "float", get_fadd, lhs, rhs);
impl_binary_inst!(Sub, "i32", get_sub, lhs, rhs);
impl_binary_inst!(FSub, "float", get_fsub, lhs, rhs);
impl_binary_inst!(Mul, "i32", get_mul, lhs, rhs);
impl_binary_inst!(FMul, "float", get_fmul, lhs, rhs);
impl_binary_inst!(UDiv, "i32", get_udiv, lhs, rhs);
impl_binary_inst!(SDiv, "i32", get_sdiv, lhs, rhs);
impl_binary_inst!(FDiv, "float", get_fdiv, lhs, rhs);
impl_binary_inst!(URem, "i32", get_urem, lhs, rhs);
impl_binary_inst!(SRem, "i32", get_srem, lhs, rhs);
impl_binary_inst!(Shl, "i32", get_shl, value, shiftamt);
impl_binary_inst!(LShr, "i32", get_lshr, value, shiftamt);
impl_binary_inst!(AShr, "i32", get_ashr, value, shiftamt);
impl_binary_inst!(And, "i32", get_and, lhs, rhs);
impl_binary_inst!(Or, "i32", get_or, lhs, rhs);
impl_binary_inst!(Xor, "i32", get_xor, lhs, rhs);
