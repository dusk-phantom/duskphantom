use crate::errors::MiddelError;
use crate::frontend::{Expr, Type};
use crate::middle::ir::Constant;
use crate::middle::irgen::util;

/// Convert a type to its default constant
pub fn type_to_const(ty: &Type) -> Result<Vec<Constant>, MiddelError> {
    match ty {
        Type::Void => Err(MiddelError::GenError),
        Type::Int32 => Ok(vec![Constant::Int(0)]),
        Type::Float32 => Ok(vec![Constant::Float(0.0)]),
        Type::String => Err(MiddelError::GenError),
        Type::Char => Err(MiddelError::GenError),
        Type::Boolean => Ok(vec![Constant::Bool(false)]),
        Type::Pointer(_) => Err(MiddelError::GenError),
        Type::Array(ty, num) => Ok(util::repeat_vec(type_to_const(ty)?, *num)),
        Type::Function(_, _) => Err(MiddelError::GenError),
        Type::Enum(_) => Err(MiddelError::GenError),
        Type::Union(_) => Err(MiddelError::GenError),
        Type::Struct(_) => Err(MiddelError::GenError),
    }
}

/// Convert a constant expression to a constant
pub fn expr_to_const(val: &Expr) -> Result<Vec<Constant>, MiddelError> {
    match val {
        Expr::Var(_) => Err(MiddelError::GenError),
        Expr::Pack(pack) => pack
            .iter()
            // Convert inner expression to constant value
            .map(expr_to_const)
            // Collect as a large result
            .collect::<Result<Vec<Vec<_>>, _>>()
            // Flatten inner vec
            .map(|v| v.into_iter().flatten().collect()),
        Expr::Map(_) => Err(MiddelError::GenError),
        Expr::Index(_, _) => Err(MiddelError::GenError),
        Expr::Field(_, _) => Err(MiddelError::GenError),
        Expr::Select(_, _) => Err(MiddelError::GenError),
        Expr::Int32(i) => Ok(vec![Constant::Int(*i)]),
        Expr::Float32(f) => Ok(vec![Constant::Float(*f)]),
        Expr::String(_) => Err(MiddelError::GenError),
        Expr::Char(_) => Err(MiddelError::GenError),
        Expr::Bool(b) => Ok(vec![Constant::Bool(*b)]),
        Expr::Call(_, _) => Err(MiddelError::GenError),
        Expr::Unary(_, _) => Err(MiddelError::GenError),
        Expr::Binary(_, _, _) => Err(MiddelError::GenError),
        Expr::Conditional(_, _, _) => Err(MiddelError::GenError),
    }
}
