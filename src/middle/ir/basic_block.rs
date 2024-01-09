use super::*;
pub struct BasicBlock {
    name: String,
}

impl BasicBlock {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
