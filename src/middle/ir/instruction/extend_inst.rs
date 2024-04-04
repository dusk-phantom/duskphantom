use std::fmt::Formatter;

use super::*;

impl IRBuilder {
    pub fn get_zext(&mut self, src: Operand) -> InstPtr {
        let mut inst = self.new_instruction(Box::new(ZextTo {
            manager: InstManager::new(ValueType::Int),
        }));
        unsafe {
            inst.get_manager_mut().add_operand(src);
        }
        inst
    }

    pub fn get_sext(&mut self, src: Operand) -> InstPtr {
        let mut inst = self.new_instruction(Box::new(SextTo {
            manager: InstManager::new(ValueType::Int),
        }));
        unsafe {
            inst.get_manager_mut().add_operand(src);
        }
        inst
    }
}

/// zero extend  bool to  int
pub struct ZextTo {
    manager: InstManager,
}

impl ZextTo {
    /// Get the operand which will be zero extended
    pub fn get_src(&self) -> &Operand {
        &self.get_operand()[0]
    }

    /// Set the operand which will be zero extended
    pub unsafe fn set_src(&mut self, src: Operand) {
        self.manager.set_operand(0, src);
    }
}

impl Display for ZextTo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "%zext_{}", self.get_id())
    }
}

impl Instruction for ZextTo {
    gen_common_code!(ZextTo, ZextTo);
    fn gen_llvm_ir(&self) -> String {
        format!("{} = zext i1 {} to i32", self, self.get_src())
    }
}

/// Sign extend  bool to  int
pub struct SextTo {
    manager: InstManager,
}

impl SextTo {
    /// Get the operand which will be sign extended
    pub fn get_src(&self) -> &Operand {
        &self.get_operand()[0]
    }
    /// Set the operand which will be sign extended
    pub unsafe fn set_src(&mut self, src: Operand) {
        self.manager.set_operand(0, src);
    }
}

impl Display for SextTo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "%sext_{}", self.get_id())
    }
}

impl Instruction for SextTo {
    gen_common_code!(SextTo, SextTo);
    fn gen_llvm_ir(&self) -> String {
        format!("{} = sext i1 {} to i32", self, self.get_src())
    }
}
