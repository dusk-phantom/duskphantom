use super::*;

pub type InstPtr = ObjPtr<Box<dyn Instruction>>;
pub trait Instruction {}

enum InstType {
    /// 无实际意义，平时操作不会拿到这个指令
    Head,
}

pub mod head;
