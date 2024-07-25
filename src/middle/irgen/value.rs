use std::collections::VecDeque;

use anyhow::{anyhow, Context, Result};

use crate::context;
use crate::middle::ir::instruction::misc_inst::{FCmpOp, ICmpOp};
use crate::middle::ir::{Constant, Operand, ValueType};
use crate::middle::irgen::function_kit::FunctionKit;

/// A value can be an operand, or a pointer to an operand.
/// An operand can not be assigned to, while a pointed value can
#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    ReadOnly(Operand),
    ReadWrite(Operand),

    /// An array of values.
    /// Values in the array must all have the same type.
    Array(Vec<Value>),
}

/// A value can be allocated with type and kit
pub fn alloc(ty: ValueType, kit: &mut FunctionKit) -> Value {
    // Add instruction to exit
    let inst = kit.program.mem_pool.get_alloca(ty, 1);
    kit.exit.unwrap().push_back(inst);
    Value::ReadWrite(inst.into())
}

/// A constant can be converted to a value
impl From<Constant> for Value {
    fn from(val: Constant) -> Self {
        Value::ReadOnly(Operand::Constant(val))
    }
}

/// Convenient operations on a value
impl Value {
    /// Get the type of value
    pub fn get_type(&self) -> ValueType {
        match self {
            Value::ReadOnly(operand) => operand.get_type(),
            Value::ReadWrite(pointer) => match pointer.get_type() {
                // Inside `ReadWrite` is the pointer to given value,
                // we're just getting type of the value
                ValueType::Pointer(ty) => *ty,
                _ => panic!("invalid pointer generated, whose content is not a pointer"),
            },
            Value::Array(operand) => {
                // Inside `Array` is an array of values,
                // we're getting type of the array
                let ty = operand[0].get_type();
                ValueType::Array(Box::new(ty), operand.len())
            }
        }
    }

    /// Load the value as an operand without type-cast,
    /// returns the loaded operand along with it's type.
    ///
    /// The type changes to pointer when attempt to load array,
    /// i.e. `[n x element_type]*` is treated as `element_type**`,
    /// causing arrays to be passed by reference.
    pub fn load_uncast(self, kit: &mut FunctionKit) -> Result<(Operand, ValueType)> {
        let value_type = self.get_type();

        // Load uncast operand
        // If this is a read-write value, load the operand from pointer,
        // otherwise just return the operand
        match self {
            Value::ReadOnly(operand) => Ok((operand, value_type)),
            Value::ReadWrite(pointer) => {
                let (inst, loaded_type) = match value_type {
                    ValueType::Array(ref element_type, _) => {
                        // This GEP changes `[n x element_type]*` to `element_type*`
                        let inst = kit.program.mem_pool.get_getelementptr(
                            value_type.clone(),
                            pointer,
                            vec![Constant::Int(0).into(), Constant::Int(0).into()],
                        );

                        // Array can't be loaded directly, instead pass array by reference
                        // So the `value_type` is changed to reference accordingly
                        (inst, ValueType::Pointer((*element_type.clone()).into()))
                    }
                    _ => (
                        // This load changes `element_type**` to `element_type*
                        kit.program.mem_pool.get_load(value_type.clone(), pointer),
                        value_type,
                    ),
                };
                kit.exit.unwrap().push_back(inst);
                Ok((inst.into(), loaded_type))
            }
            Value::Array(_) => {
                // Array is not loadable
                Err(anyhow!("array is not loadable")).with_context(|| context!())
            }
        }
    }

    /// Load the value as an operand
    pub fn load(self, target: ValueType, kit: &mut FunctionKit) -> Result<Operand> {
        let (uncast_operand, loaded_type) = self.load_uncast(kit)?;

        // Return directly if type matches
        if loaded_type == target {
            return Ok(uncast_operand);
        }

        // Convert type if not match
        match (loaded_type, target) {
            (ValueType::Int, ValueType::Float) => {
                // Direct convert
                let inst = kit.program.mem_pool.get_itofp(uncast_operand);
                kit.exit.unwrap().push_back(inst);
                Ok(inst.into())
            }
            (ValueType::Float, ValueType::Int) => {
                // Direct convert
                let inst = kit.program.mem_pool.get_fptoi(uncast_operand);
                kit.exit.unwrap().push_back(inst);
                Ok(inst.into())
            }
            (ValueType::Bool, ValueType::Int) => {
                // Direct convert
                let inst = kit.program.mem_pool.get_zext(uncast_operand);
                kit.exit.unwrap().push_back(inst);
                Ok(inst.into())
            }
            (ValueType::Bool, ValueType::Float) => {
                // Convert to int first and then float
                let inst = kit.program.mem_pool.get_zext(uncast_operand);
                let inst = kit.program.mem_pool.get_itofp(inst.into());
                kit.exit.unwrap().push_back(inst);
                Ok(inst.into())
            }
            (ValueType::Int, ValueType::Bool) => {
                // Direct convert
                let inst = kit.program.mem_pool.get_icmp(
                    ICmpOp::Ne,
                    ValueType::Int,
                    uncast_operand,
                    Constant::Int(0).into(),
                );
                kit.exit.unwrap().push_back(inst);
                Ok(inst.into())
            }
            (ValueType::Float, ValueType::Bool) => {
                // Compare with 0.0 (NaN is treated as true)
                let inst = kit.program.mem_pool.get_fcmp(
                    FCmpOp::Une,
                    ValueType::Float,
                    uncast_operand,
                    Constant::Float(0.0).into(),
                );
                kit.exit.unwrap().push_back(inst);
                Ok(inst.into())
            }
            (ty, target) => {
                Err(anyhow!("cannot load from {} to {}", ty, target)).with_context(|| context!())
            }
        }
    }

    /// Shift the underlying pointer (if exists)
    ///
    /// Element of index is [shift by whole, shift by primary element, ...]
    ///
    /// For example, get_element_ptr([2, 8]) on a pointer to an array [n x i32]
    /// shifts it by (2 * n + 8) * sizeof i32.
    ///
    /// If current value type is pointer, it will be treated as array,
    /// i.e. `element_type**` is treated as `[n * element_type]*`,
    /// making array reference act like an array.
    pub fn getelementptr(self, kit: &mut FunctionKit, index: Vec<Operand>) -> Result<Value> {
        let value_type = self.get_type();
        match self {
            Value::ReadOnly(_) => {
                Err(anyhow!("can't GEP from operand")).with_context(|| context!())
            }
            Value::Array(_) => Err(anyhow!("cannot GEP from array")).with_context(|| context!()),
            Value::ReadWrite(pointer) => {
                match value_type {
                    // Treat pointer as array here, convert `element_type**` to `element_type*` with `load`
                    ValueType::Pointer(ref element_type) => {
                        let load = kit.program.mem_pool.get_load(value_type.clone(), pointer);
                        kit.exit.unwrap().push_back(load);

                        // Remove the first element of index array
                        let mut vec_deque = VecDeque::from(index);
                        vec_deque.pop_front();

                        // Build the rest of GEP if there is more indexes
                        if vec_deque.is_empty() {
                            Ok(Value::ReadWrite(load.into()))
                        } else {
                            let gep = kit.program.mem_pool.get_getelementptr(
                                *element_type.clone(),
                                Operand::Instruction(load),
                                vec_deque.into(),
                            );
                            kit.exit.unwrap().push_back(gep);
                            Ok(Value::ReadWrite(gep.into()))
                        }
                    }

                    // Convert `[n x element_type]*` to `element_type*` with `getelementptr`
                    _ => {
                        let gep = kit
                            .program
                            .mem_pool
                            .get_getelementptr(value_type, pointer, index);
                        kit.exit.unwrap().push_back(gep);
                        Ok(Value::ReadWrite(gep.into()))
                    }
                }
            }
        }
    }

    /// Assign a value to this value
    pub fn assign(self, kit: &mut FunctionKit, val: Value) -> Result<()> {
        let target = self.get_type();

        // If target is array, load each element separately
        if let Value::Array(arr) = val {
            // Get first sub-pointer
            let initial_ptr =
                self.getelementptr(kit, vec![Constant::Int(0).into(), Constant::Int(0).into()])?;

            // Iterate all sub-pointers
            for (i, elem) in arr.into_iter().enumerate() {
                let sub_ptr = initial_ptr
                    .clone()
                    .getelementptr(kit, vec![Constant::Int(i as i32).into()])?;

                // Assign element to sub-pointer
                sub_ptr.assign(kit, elem)?;
            }
            return Ok(());
        }

        // Otherwise load element
        match self {
            Value::ReadOnly(_) => Err(anyhow!("cannot assign operand")).with_context(|| context!()),
            Value::Array(_) => Err(anyhow!("cannot assign to array")).with_context(|| context!()),
            Value::ReadWrite(ptr) => {
                // Load operand from value first
                let op = val.load(target, kit)?;

                // Store operand to pointer
                let inst = kit.program.mem_pool.get_store(op, ptr);
                kit.exit.unwrap().push_back(inst);
                Ok(())
            }
        }
    }
}
