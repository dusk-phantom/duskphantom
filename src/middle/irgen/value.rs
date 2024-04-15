use crate::errors::MiddelError;
use crate::middle::ir::{Constant, Operand, ValueType};
use crate::middle::irgen::function_kit::FunctionKit;

/// A value can be an operand, or a pointer to an operand.
/// An operand can not be assigned to, while a pointed value can
#[derive(Clone)]
pub enum Value {
    Operand(Operand),
    Pointer(Operand),
}

/// A value can be allocated with type and kit
pub fn alloc(ty: ValueType, kit: &mut FunctionKit) -> Value {
    // Add instruction to exit
    let inst = kit.program.mem_pool.get_alloca(ty, 1);
    kit.exit.push_back(inst);
    Value::Pointer(inst.into())
}

/// A constant can be converted to a value
impl From<Constant> for Value {
    fn from(val: Constant) -> Self {
        Value::Operand(Operand::Constant(val))
    }
}

/// Convenient operations on a value
impl Value {
    /// Get the type of value
    pub fn get_type(&self) -> ValueType {
        match self {
            Value::Operand(op) => op.get_type(),
            Value::Pointer(op) => match op.get_type() {
                // Inside `Pointer` is the pointer to given value
                ValueType::Pointer(ty) => *ty,
                _ => panic!("invalid pointer generated, whose content is not a pointer"),
            },
        }
    }

    /// Load the value as an operand
    pub fn load(self, kit: &mut FunctionKit) -> Operand {
        let ty = self.get_type();
        match self {
            Value::Operand(op) => op,
            Value::Pointer(op) => {
                // Add instruction to exit
                let inst = kit.program.mem_pool.get_load(ty, op);
                kit.exit.push_back(inst);
                inst.into()
            }
        }
    }

    /// Shift the underlying pointer (if exists)
    /// Element of index is [shift by whole, shift by primary element, ...]
    /// For example, get_element_ptr([2, 8]) on a pointer to an array [n x i32]
    /// shifts it by (2 * n + 8) * sizeof i32.
    /// DO NOT FORGET THE FIRST INDEX
    pub fn get_element_ptr(
        self,
        kit: &mut FunctionKit,
        index: Vec<Operand>,
    ) -> Result<Value, MiddelError> {
        let ty = self.get_type();
        match self {
            Value::Operand(_) => Err(MiddelError::GenError),
            Value::Pointer(op) => {
                // Add instruction to exit
                let inst = kit.program.mem_pool.get_getelementptr(ty, op, index);
                kit.exit.push_back(inst);

                // Construct new value
                // TODO Type of pointer should be shrunk (as "get element" states)
                Ok(Value::Pointer(inst.into()))
            }
        }
    }

    /// Assign an operand to this value
    pub fn assign(self, kit: &mut FunctionKit, op: Operand) -> Result<(), MiddelError> {
        match self {
            Value::Operand(_) => Err(MiddelError::GenError),
            Value::Pointer(ptr) => {
                // Add instruction to exit
                let inst = kit.program.mem_pool.get_store(op, ptr);
                kit.exit.push_back(inst);
                Ok(())
            }
        }
    }
}
