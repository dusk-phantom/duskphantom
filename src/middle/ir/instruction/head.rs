use super::*;
use crate::impl_instruction_common_methods;
pub struct Head {
    manager: InstManager,
}

impl Head {
    pub fn new() -> Self {
        Self {
            manager: InstManager::new(),
        }
    }
}

impl_instruction_common_methods!(Head, Head);
