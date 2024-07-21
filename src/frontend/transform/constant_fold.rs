use anyhow::Result;

use crate::{
    frontend::{BinaryOp, Expr, Type, UnaryOp},
    utils::frame_map::FrameMap,
};

impl Expr {
    fn to_i32(&self) -> Result<i32> {
        match self {
            Expr::Int(x) => Ok(*x),
            Expr::Float(x) => Ok(*x as i32),
            _ => Err(anyhow::anyhow!("Cannot cast to i32")),
        }
    }

    fn to_f32(&self) -> Result<f32> {
        match self {
            Expr::Int(x) => Ok(*x as f32),
            Expr::Float(x) => Ok(*x),
            _ => Err(anyhow::anyhow!("Cannot cast to f32")),
        }
    }
}

impl From<i32> for Expr {
    fn from(i: i32) -> Self {
        Self::Int(i)
    }
}

impl From<f32> for Expr {
    fn from(fl: f32) -> Self {
        Self::Float(fl)
    }
}

impl From<bool> for Expr {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

pub fn fold_expr(expr: Expr, env: &FrameMap<String, Expr>, expr_type: &Type) -> Result<Expr> {
    match expr_type {
        Type::Array(element_type, count) => {
            let arr = fold_array(expr, env, element_type)?;
            Ok(Expr::Array(arr))
        }
        Type::Int => {
            let x = fold_i32(expr, env)?;
            Ok(Expr::Int(x))
        }
        Type::Float => {
            let x = fold_f32(expr, env)?;
            Ok(Expr::Float(x))
        }
        _ => Ok(expr),
    }
}

pub fn fold_array(
    expr: Expr,
    env: &FrameMap<String, Expr>,
    indexes: Vec<usize>,
    element_type: &Type,
) -> Result<Expr> {
    match expr {
        Expr::Var(id) => {
            let Some(val) = env.get(&id) else {
                return Err(anyhow::anyhow!("Variable not found"));
            };
            Ok(val.clone())
        }
        Expr::Array(arr) => Ok(arr),
        Expr::Map(_) => todo!(),
        Expr::Index(arr, ix) => {
            let array_type = Type::Array(element_type.clone().into(), Expr::Int(0).into());
            let arr = fold_array(*arr, env, &array_type)?;
            let ix = ix.to_i32()?;
            if ix < 0 || ix as usize >= arr.len() {
                return Ok(vec![]);
            }
            arr[ix as usize].to_array()
        }
        Expr::Field(_, _) => todo!(),
        Expr::Select(_, _) => todo!(),
        Expr::Int(_) => todo!(),
        Expr::Float(_) => todo!(),
        Expr::String(_) => todo!(),
        Expr::Char(_) => todo!(),
        Expr::Bool(_) => todo!(),
        Expr::Call(_, _) => todo!(),
        Expr::Unary(_, _) => todo!(),
        Expr::Binary(_, _) => todo!(),
        Expr::Conditional(_, _, _) => todo!(),
    }
}

pub fn fold_i32(expr: Expr, env: &FrameMap<String, Expr>) -> Result<i32> {
    match expr {
        Expr::Var(id) => {
            let Some(val) = env.get(id) else {
                return Err(anyhow::anyhow!("Variable not found"));
            };
            val.to_i32()
        }
        Expr::Array(_) => todo!(),
        Expr::Map(_) => todo!(),
        Expr::Index(arr, ix) => {
            let ix = fold_i32(*ix, env)?;
            fold_array(arr, env, vec![ix as usize], &Type::Int)
        }
        Expr::Field(_, _) => todo!(),
        Expr::Select(_, _) => todo!(),
        Expr::Int(x) => Ok(*x),
        Expr::Float(x) => Ok(*x as i32),
        Expr::String(_) => todo!(),
        Expr::Char(x) => Ok(*x as i32),
        Expr::Bool(x) => Ok(*x as i32),
        Expr::Call(_, _) => todo!(),
        Expr::Unary(op, expr) => {
            let x = fold_i32(expr, env)?;
            match op {
                UnaryOp::Neg => Ok(-x),
                UnaryOp::Pos => Ok(x),
                UnaryOp::Not => Ok(if x == 0 { 1 } else { 0 }),
                _ => todo!(),
            }
        }
        Expr::Binary(head, tail) => {
            let mut x = fold_i32(head, env)?;
            for (op, expr) in tail {
                let y = fold_i32(expr, env)?;
                match op {
                    BinaryOp::Add => x += y,
                    BinaryOp::Sub => x -= y,
                    BinaryOp::Mul => x *= y,
                    BinaryOp::Div => x /= y,
                    BinaryOp::Mod => x %= y,
                    BinaryOp::Shr => x >>= y,
                    BinaryOp::Shl => x <<= y,
                    BinaryOp::BitAnd => x &= y,
                    BinaryOp::BitOr => x |= y,
                    BinaryOp::BitXor => x ^= y,
                    BinaryOp::Gt => x = if x > y { 1 } else { 0 },
                    BinaryOp::Lt => x = if x < y { 1 } else { 0 },
                    BinaryOp::Ge => x = if x >= y { 1 } else { 0 },
                    BinaryOp::Le => x = if x <= y { 1 } else { 0 },
                    BinaryOp::Eq => x = if x == y { 1 } else { 0 },
                    BinaryOp::Ne => x = if x != y { 1 } else { 0 },
                    BinaryOp::And => x = if x != 0 && y != 0 { 1 } else { 0 },
                    BinaryOp::Or => x = if x != 0 || y != 0 { 1 } else { 0 },
                };
            }
            Ok(x)
        }
        Expr::Conditional(_, _, _) => todo!(),
    }
}

pub fn fold_f32(expr: Expr, env: &FrameMap<String, Expr>) -> Result<f32> {
    match expr {
        Expr::Var(id) => {
            let Some(val) = env.get(&id) else {
                return Err(anyhow::anyhow!("Variable not found"));
            };
            val.to_f32()
        }
        Expr::Array(_) => todo!(),
        Expr::Map(_) => todo!(),
        Expr::Index(arr, ix) => {
            let arr = fold_array(arr, env)?;
            let ix = ix.to_i32()?;
            if ix < 0 || ix as usize >= arr.len() {
                return Err(anyhow::anyhow!("Index out of bounds"));
            }
            fold_f32(&arr[ix as usize], env)
        }
        Expr::Field(_, _) => todo!(),
        Expr::Select(_, _) => todo!(),
        Expr::Int(x) => Ok(*x as f32),
        Expr::Float(x) => Ok(*x),
        Expr::String(_) => todo!(),
        Expr::Char(x) => Ok(*x as i32 as f32),
        Expr::Bool(x) => Ok(*x as i32 as f32),
        Expr::Call(_, _) => todo!(),
        Expr::Unary(op, expr) => {
            let x = fold_f32(expr, env)?;
            match op {
                UnaryOp::Neg => Ok(-x),
                UnaryOp::Pos => Ok(x),
                UnaryOp::Not => Ok(if x == 0.0 { 1.0 } else { 0.0 }),
                _ => todo!(),
            }
        }
        Expr::Binary(head, tail) => {
            let mut x = fold_f32(head, env)?;
            for (op, expr) in tail {
                let y = fold_f32(expr, env)?;
                match op {
                    BinaryOp::Add => x += y,
                    BinaryOp::Sub => x -= y,
                    BinaryOp::Mul => x *= y,
                    BinaryOp::Div => x /= y,
                    BinaryOp::Mod => x %= y,
                    BinaryOp::Shr => return Err(anyhow::anyhow!("Cannot shift float")),
                    BinaryOp::Shl => return Err(anyhow::anyhow!("Cannot shift float")),
                    BinaryOp::BitAnd => return Err(anyhow::anyhow!("Cannot bitwise and float")),
                    BinaryOp::BitOr => return Err(anyhow::anyhow!("Cannot bitwise or float")),
                    BinaryOp::BitXor => return Err(anyhow::anyhow!("Cannot bitwise xor float")),
                    BinaryOp::Gt => x = if x > y { 1.0 } else { 0.0 },
                    BinaryOp::Lt => x = if x < y { 1.0 } else { 0.0 },
                    BinaryOp::Ge => x = if x >= y { 1.0 } else { 0.0 },
                    BinaryOp::Le => x = if x <= y { 1.0 } else { 0.0 },
                    BinaryOp::Eq => x = if x == y { 1.0 } else { 0.0 },
                    BinaryOp::Ne => x = if x != y { 1.0 } else { 0.0 },
                    BinaryOp::And => x = if x != 0.0 && y != 0.0 { 1.0 } else { 0.0 },
                    BinaryOp::Or => x = if x != 0.0 || y != 0.0 { 1.0 } else { 0.0 },
                };
            }
            Ok(x)
        }
        Expr::Conditional(_, _, _) => todo!(),
    }
}
