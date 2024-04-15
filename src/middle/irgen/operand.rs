use crate::errors::MiddelError;
use crate::middle::ir::{Operand, ValueType};
use crate::middle::irgen::function_kit::FunctionKit;

/// Convenient methods for operand
impl Operand {
    /// Convert the type of operand to another
    pub fn conv(self, ty: ValueType, kit: &mut FunctionKit) -> Result<Operand, MiddelError> {
        let from_ty = self.get_type();
        if from_ty == ty {
            return Ok(self);
        }
        match (from_ty, ty) {
            (ValueType::Int, ValueType::Float) => {
                // Direct convert
                let inst = kit.program.mem_pool.get_itofp(self);
                kit.exit.push_back(inst);
                Ok(inst.into())
            }
            (ValueType::Float, ValueType::Int) => {
                // Direct convert
                let inst = kit.program.mem_pool.get_fptoi(self);
                kit.exit.push_back(inst);
                Ok(inst.into())
            }
            (ValueType::Bool, ValueType::Int) => {
                // Direct convert
                let inst = kit.program.mem_pool.get_zext(self);
                kit.exit.push_back(inst);
                Ok(inst.into())
            }
            (ValueType::Bool, ValueType::Float) => {
                // Convert to int first and then float
                let inst = kit.program.mem_pool.get_zext(self);
                let inst = kit.program.mem_pool.get_itofp(inst.into());
                kit.exit.push_back(inst);
                Ok(inst.into())
            }
            _ => Err(MiddelError::GenError),
        }
    }

    /// Unify the types of two operands
    pub fn unify(a: Self, b: Self, kit: &mut FunctionKit) -> Result<(Self, Self), MiddelError> {
        let a_ty = a.get_type();
        let b_ty = b.get_type();
        let max_ty = a_ty.max_with(&b_ty)?;
        Ok((a.conv(max_ty.clone(), kit)?, b.conv(max_ty, kit)?))
    }
}
