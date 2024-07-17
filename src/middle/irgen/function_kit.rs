use crate::middle;
use crate::middle::ir::{BBPtr, FunPtr, ValueType};
use crate::middle::irgen::value::Value;
use std::collections::HashMap;

use super::program_kit::ProgramKit;

/// Kit for translating a function to middle IR
pub struct FunctionKit<'a> {
    pub env: &'a mut Vec<HashMap<String, Value>>,
    pub fun_env: &'a mut Vec<HashMap<String, FunPtr>>,
    pub program: &'a mut middle::Program,
    pub exit: Option<BBPtr>,
    pub break_to: Option<BBPtr>,
    pub continue_to: Option<BBPtr>,
    pub return_to: BBPtr,
    pub return_value: Option<Value>,
    pub return_type: ValueType,
    pub counter: &'a mut usize,
}

/// Context for FunctionKit
pub struct FunctionContext<'a> {
    pub env: &'a mut Vec<HashMap<String, Value>>,
    pub fun_env: &'a mut Vec<HashMap<String, FunPtr>>,
    pub program: &'a mut middle::Program,
    pub counter: &'a mut usize,
}

/// Routing for FunctionKit
pub struct FunctionRouting {
    pub exit: Option<BBPtr>,
    pub break_to: Option<BBPtr>,
    pub continue_to: Option<BBPtr>,
    pub return_to: BBPtr,
    pub return_value: Option<Value>,
    pub return_type: ValueType,
}

/// A function kit can generate statements
impl<'a> FunctionKit<'a> {
    /// Create a new function kit
    /// Environment will remain the same after drop
    pub fn new(ctx: FunctionContext<'a>, routing: FunctionRouting) -> Self {
        // Initialize a new frame of environment
        ctx.env.push(HashMap::new());
        ctx.fun_env.push(HashMap::new());

        // Create function kit
        FunctionKit {
            env: ctx.env,
            fun_env: ctx.fun_env,
            program: ctx.program,
            exit: routing.exit,
            break_to: routing.break_to,
            continue_to: routing.continue_to,
            return_to: routing.return_to,
            return_value: routing.return_value,
            return_type: routing.return_type,
            counter: ctx.counter,
        }
    }

    /// Generate a new function kit from existing one
    pub fn gen_function_kit(
        &mut self,
        exit: Option<BBPtr>,
        break_to: Option<BBPtr>,
        continue_to: Option<BBPtr>,
    ) -> FunctionKit {
        FunctionKit::new(
            FunctionContext {
                env: self.env,
                fun_env: self.fun_env,
                program: self.program,
                counter: self.counter,
            },
            FunctionRouting {
                exit,
                break_to,
                continue_to,
                return_to: self.return_to,
                return_value: self.return_value.clone(),
                return_type: self.return_type.clone(),
            },
        )
    }

    /// Get from environment
    pub fn get_env(&self, name: &str) -> Option<Value> {
        for frame in self.env.iter().rev() {
            if let Some(val) = frame.get(name) {
                return Some(val.clone());
            }
        }
        None
    }

    /// Insert to environment
    pub fn insert_env(&mut self, name: String, value: Value) {
        self.env.last_mut().unwrap().insert(name, value);
    }

    /// Get from func environment
    pub fn get_fun_env(&self, name: &str) -> Option<FunPtr> {
        for frame in self.fun_env.iter().rev() {
            if let Some(val) = frame.get(name) {
                return Some(*val);
            }
        }
        None
    }

    /// Generate a new program kit to insert new constants.
    ///
    /// Note: the generated program kit contains NO environment!
    /// If environment is needed when inserting constants, the function needs changes.
    pub fn gen_program_kit(&mut self) -> ProgramKit {
        ProgramKit {
            env: self.env,
            fun_env: self.fun_env,
            program: self.program,
        }
    }

    /// Generate a unique basic block name
    pub fn unique_name(&mut self, base: &str) -> String {
        let name = format!("{}{}", base, self.counter);
        *self.counter += 1;
        name
    }
}

/// Delete current environment frame when the function kit is dropped
impl<'a> Drop for FunctionKit<'a> {
    fn drop(&mut self) {
        self.env.pop();
        self.fun_env.pop();
    }
}
