use super::*;

pub type InstPtr = ObjPtr<Box<dyn Instruction>>;
pub trait Instruction {}
