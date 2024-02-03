use super::*;
use crate::impl_BinaryInst;

/// impl for binary operation and bitwise binary_inst
pub trait BinaryInst {
    fn get_lhs(&self) -> InstPtr;
    fn set_lhs(&mut self, lhs: InstPtr);
    fn get_rhs(&self) -> InstPtr;
    fn set_rhs(&mut self, rhs: InstPtr);
}

pub struct Add {
    manager: InstManager,
}
impl_BinaryInst!(Add);
impl Instruction for Add {
    gen_common_code!(Add, Add);
    #[inline]
    fn gen_llvm_ir(&self) -> String {
        format!(
            "{} = add nsw i32 {}, {}",
            self.get_id(),
            self.get_lhs().get_id(),
            self.get_rhs().get_id()
        )
    }
}
