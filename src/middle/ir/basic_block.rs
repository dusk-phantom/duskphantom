use super::*;
pub struct BasicBlock {
    name: String,
    context: ObjPtr<ContextArena>,
}

impl BasicBlock {
    pub fn new(name: String, context: &ContextArena) -> Self {
        Self {
            name,
            context: ObjPtr::new(context),
        }
    }
}
