use crate::middle;
use crate::middle::ir::{BBPtr, FunPtr, ValueType};
use crate::middle::irgen::value::Value;
use crate::utils::frame_map::FrameMap;

/// Kit for translating a function to middle IR
pub struct FunctionKit<'a> {
    pub env: FrameMap<'a, String, Value>,
    pub fun_env: FrameMap<'a, String, FunPtr>,
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
    pub env: FrameMap<'a, String, Value>,
    pub fun_env: FrameMap<'a, String, FunPtr>,
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
                env: self.env.branch(),
                fun_env: self.fun_env.branch(),
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

    /// Generate a unique basic block name
    pub fn unique_name(&mut self, base: &str) -> String {
        let name = format!("{}{}", base, self.counter);
        *self.counter += 1;
        name
    }
}
