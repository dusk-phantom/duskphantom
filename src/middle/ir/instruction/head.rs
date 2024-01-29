use super::*;
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

impl Instruction for Head {
    gen_common_code!(Head, Head);
    fn gen_llvm_ir(&self) -> String {
        String::new()
    }
}
