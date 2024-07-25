use anyhow::{anyhow, Context, Result};

use crate::{
    context,
    frontend::Type,
    middle::ir::{Constant, ValueType},
};

use super::gen_const::gen_const;

/// Translate a frontend type to IR value type
pub fn gen_type(ty: &Type) -> Result<ValueType> {
    match ty {
        Type::Void => Ok(ValueType::Void),
        Type::Int => Ok(ValueType::Int),
        Type::Float => Ok(ValueType::Float),
        Type::Bool => Ok(ValueType::Bool),
        Type::Pointer(ty) => Ok(ValueType::Pointer(Box::new(gen_type(ty)?))),
        Type::Array(ty, index_expr) => {
            let index_constant = gen_const(index_expr)?;
            let Constant::Int(index) = index_constant else {
                return Err(anyhow!("index is not an integer")).with_context(|| context!());
            };
            Ok(ValueType::Array(Box::new(gen_type(ty)?), index as usize))
        }
        _ => Err(anyhow!("type {:?} can't translate to middle", ty)).with_context(|| context!()),
    }
}
