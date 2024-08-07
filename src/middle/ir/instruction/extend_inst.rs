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

    pub fn get_itofp(&mut self, src: Operand) -> InstPtr {
        let mut inst = self.new_instruction(Box::new(ItoFp {
            manager: InstManager::new(ValueType::Float),
        }));
        unsafe {
            inst.get_manager_mut().add_operand(src);
        }
        inst
    }

    pub fn get_fptoi(&mut self, src: Operand) -> InstPtr {
        let mut inst = self.new_instruction(Box::new(FpToI {
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

    /// # Safety
    ///
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
    fn copy_self(&self) -> Box<dyn Instruction> {
        Box::new(ZextTo {
            manager: InstManager::new(ValueType::Int),
        })
    }
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
    ///  # Safety
    ///
    ///  Set the operand which will be sign extended
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
    fn copy_self(&self) -> Box<dyn Instruction> {
        Box::new(SextTo {
            manager: InstManager::new(ValueType::Int),
        })
    }
    fn gen_llvm_ir(&self) -> String {
        format!("{} = sext i1 {} to i32", self, self.get_src())
    }
}

pub struct ItoFp {
    manager: InstManager,
}

impl ItoFp {
    /// Get the operand which will be converted
    pub fn get_src(&self) -> &Operand {
        &self.get_operand()[0]
    }
    /// # Safety
    ///
    /// Set the operand which will be converted
    pub unsafe fn set_src(&mut self, src: Operand) {
        self.manager.set_operand(0, src);
    }
}

impl Display for ItoFp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "%itofp_{}", self.get_id())
    }
}

impl Instruction for ItoFp {
    gen_common_code!(ItoFp, ItoFp);
    fn copy_self(&self) -> Box<dyn Instruction> {
        Box::new(ItoFp {
            manager: InstManager::new(ValueType::Float),
        })
    }
    fn gen_llvm_ir(&self) -> String {
        format!("{} = sitofp i32 {} to float", self, self.get_src())
    }
}

pub struct FpToI {
    manager: InstManager,
}

impl FpToI {
    /// Get the operand which will be converted
    pub fn get_src(&self) -> &Operand {
        &self.get_operand()[0]
    }
    /// # Safety
    ///
    /// Set the operand which will be converted
    pub unsafe fn set_src(&mut self, src: Operand) {
        self.manager.set_operand(0, src);
    }
}

impl Display for FpToI {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "%fptoi_{}", self.get_id())
    }
}

impl Instruction for FpToI {
    gen_common_code!(FpToI, FpToI);
    fn copy_self(&self) -> Box<dyn Instruction> {
        Box::new(FpToI {
            manager: InstManager::new(ValueType::Int),
        })
    }
    fn gen_llvm_ir(&self) -> String {
        format!("{} = fptosi float {} to i32", self, self.get_src())
    }
}
