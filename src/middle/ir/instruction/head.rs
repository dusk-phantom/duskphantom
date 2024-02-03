use super::*;
pub struct Head {
    manager: InstManager,
}

impl IRBuilder {
    pub fn new_head(&mut self) -> InstPtr {
        let head = Head {
            manager: InstManager::new(self.new_inst_id()),
        };
        self.new_instruction(Box::new(head))
    }
}

impl Instruction for Head {
    gen_common_code!(Head, Head);
    #[inline]
    fn gen_llvm_ir(&self) -> String {
        String::new()
    }
}
