use super::*;
pub struct Head {
    manager: InstManager,
}

impl IRBuilder {
    pub fn new_head(&mut self) -> InstPtr {
        self.new_instruction(Box::new(Head {
            manager: InstManager::new(),
        }))
    }
}

impl Instruction for Head {
    gen_common_code!(Head, Head);
    #[inline]
    fn gen_llvm_ir(&self) -> String {
        String::new()
    }
}

impl Display for Head {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}