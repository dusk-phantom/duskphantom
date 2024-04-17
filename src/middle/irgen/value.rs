use crate::errors::MiddelError;
use crate::middle::ir::instruction::misc_inst::{FCmpOp, ICmpOp};
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

    /// Load the value as a constant
    pub fn constant(self) -> Result<Constant, MiddelError> {
        match self {
            Value::Operand(Operand::Constant(c)) => Ok(c),
            _ => Err(MiddelError::GenError),
        }
    }

    /// Load the value as an operand
    pub fn load(self, target: ValueType, kit: &mut FunctionKit) -> Result<Operand, MiddelError> {
        // Load raw
        let ty = self.get_type();
        let raw = match self {
            Value::Operand(op) => op,
            Value::Pointer(op) => {
                // Add instruction to exit
                let inst = kit.program.mem_pool.get_load(ty.clone(), op);
                kit.exit.push_back(inst);
                inst.into()
            }
        };

        // Return directly if type matches
        if ty == target {
            return Ok(raw);
        }

        // Convert type if not match
        match (ty, target) {
            (ValueType::Int, ValueType::Float) => {
                // Direct convert
                let inst = kit.program.mem_pool.get_itofp(raw);
                kit.exit.push_back(inst);
                Ok(inst.into())
            }
            (ValueType::Float, ValueType::Int) => {
                // Direct convert
                let inst = kit.program.mem_pool.get_fptoi(raw);
                kit.exit.push_back(inst);
                Ok(inst.into())
            }
            (ValueType::Bool, ValueType::Int) => {
                // Direct convert
                let inst = kit.program.mem_pool.get_zext(raw);
                kit.exit.push_back(inst);
                Ok(inst.into())
            }
            (ValueType::Bool, ValueType::Float) => {
                // Convert to int first and then float
                let inst = kit.program.mem_pool.get_zext(raw);
                let inst = kit.program.mem_pool.get_itofp(inst.into());
                kit.exit.push_back(inst);
                Ok(inst.into())
            }
            (ValueType::Int, ValueType::Bool) => {
                // Direct convert
                let inst = kit.program.mem_pool.get_icmp(
                    ICmpOp::Ne,
                    ValueType::Int,
                    raw,
                    Constant::Int(0).into(),
                );
                kit.exit.push_back(inst);
                Ok(inst.into())
            }
            (ValueType::Float, ValueType::Bool) => {
                // Compare with 0.0 (NaN is treated as true)
                let inst = kit.program.mem_pool.get_fcmp(
                    FCmpOp::Une,
                    ValueType::Float,
                    raw,
                    Constant::Float(0.0).into(),
                );
                kit.exit.push_back(inst);
                Ok(inst.into())
            }
            _ => Err(MiddelError::GenError),
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
