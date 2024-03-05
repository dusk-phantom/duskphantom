use super::*;

impl IRBuilder {
    /// Create a new `IntConst` instruction.
    /// The `IntConst` instruction is used to create a constant integer value.
    ///
    /// # Arguments
    /// * `value` - The value of the constant integer
    ///
    /// # Return
    /// The pointer to the `IntConst` instruction
    pub fn get_int_const(&mut self, value: i64) -> InstPtr {
        self.new_instruction(Box::new(IntConst {
            value,
            manager: InstManager::new(),
        }))
    }

    /// Create a new `FloatConst` instruction.
    /// The `FloatConst` instruction is used to create a constant float value.
    ///
    /// # Arguments
    /// * `value` - The value of the constant float
    ///
    /// # Return
    /// The pointer to the `FloatConst` instruction
    pub fn get_float_const(&mut self, value: f64) -> InstPtr {
        self.new_instruction(Box::new(FloatConst {
            value,
            manager: InstManager::new(),
        }))
    }
}

pub struct IntConst {
    pub value: i64,
    manager: InstManager,
}

impl Display for IntConst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Instruction for IntConst {
    gen_common_code!(IntConst, IntConst);
    fn gen_llvm_ir(&self) -> String {
        "".to_string()
    }
}

pub struct FloatConst {
    pub value: f64,
    manager: InstManager,
}

impl Display for FloatConst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Instruction for FloatConst {
    gen_common_code!(FloatConst, FloatConst);
    fn gen_llvm_ir(&self) -> String {
        "".to_string()
    }
}
