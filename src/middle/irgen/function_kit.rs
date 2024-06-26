use crate::middle;
use crate::middle::ir::{BBPtr, FunPtr, ValueType};
use crate::middle::irgen::value::Value;
use std::collections::HashMap;

/// Kit for translating a function to middle IR
pub struct FunctionKit<'a> {
    pub env: HashMap<String, Value>,
    pub fun_env: HashMap<String, FunPtr>,
    pub program: &'a mut middle::Program,
    pub exit: Option<BBPtr>,
    pub break_to: Option<BBPtr>,
    pub continue_to: Option<BBPtr>,
    pub return_type: ValueType,
    pub counter: &'a mut usize,
}

/// A function kit can generate statements
impl<'a> FunctionKit<'a> {
    /// Generate a new function kit
    pub fn gen_function_kit(
        &mut self,
        exit: BBPtr,
        break_to: Option<BBPtr>,
        continue_to: Option<BBPtr>,
    ) -> FunctionKit {
        FunctionKit {
            program: self.program,
            env: self.env.clone(),
            fun_env: self.fun_env.clone(),
            exit: Some(exit),
            break_to,
            continue_to,
            return_type: self.return_type.clone(),
            counter: self.counter,
        }
    }

    /// Generate a unique basic block name
    pub fn unique_name(&mut self, base: &str) -> String {
        let name = format!("{}{}", base, self.counter);
        *self.counter += 1;
        name
    }
}
