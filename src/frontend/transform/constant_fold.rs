use anyhow::{anyhow, Context, Result};

use crate::{
    context,
    frontend::{BinaryOp, Decl, Expr, Program, Type, UnaryOp},
    utils::frame_map::FrameMap,
};

#[allow(unused)]
pub fn optimize_program(program: &mut Program) -> Result<()> {
    for decl in program.module {
        match decl {
            Decl::Const(_, _, _) => todo!(),
            Decl::Var(_, _, _) => todo!(),
            Decl::Stack(_) => todo!(),
            Decl::Func(_, _, _) => todo!(),
            Decl::Enum(_, _) => todo!(),
            Decl::Union(_, _) => todo!(),
            Decl::Struct(_, _) => todo!(),
        }
    }
    Ok(())
}

impl Expr {
    fn to_i32(&self) -> Result<i32> {
        match self {
            Expr::Int(x) => Ok(*x),
            Expr::Float(x) => Ok(*x as i32),
            _ => Err(anyhow!("Cannot cast to i32")),
        }
    }

    fn to_f32(&self) -> Result<f32> {
        match self {
            Expr::Int(x) => Ok(*x as f32),
            Expr::Float(x) => Ok(*x),
            _ => Err(anyhow!("Cannot cast to f32")),
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

/// Fold an expression to constant.
pub fn fold_expr(expr: &Expr, env: &FrameMap<String, Expr>, expr_type: &Type) -> Result<Expr> {
    match expr_type {
        Type::Array(element_type, _) => {
            let arr = fold_array(expr, env, element_type)?;
            Ok(arr)
        }
        Type::Int => {
            let x = fold_i32(expr, env)?;
            Ok(Expr::Int(x))
        }
        Type::Float => {
            let x = fold_f32(expr, env)?;
            Ok(Expr::Float(x))
        }
        _ => Err(anyhow!("cannot fold an instance of {:?}", expr_type)).with_context(|| context!()),
    }
}

/// Fold an indexed expression to constant.
pub fn fold_indexed(
    expr: &Expr,
    env: &FrameMap<String, Expr>,
    mut indexes: Vec<usize>,
    expr_type: &Type,
) -> Result<Expr> {
    // If not indexed, fallback to regular expression fold
    if indexes.is_empty() {
        return fold_expr(expr, env, expr_type);
    }

    // Expression is indexed
    match expr {
        Expr::Var(id) => {
            let Some(val) = env.get(id) else {
                return Err(anyhow!("Variable not found"));
            };

            // Although val is already folded, we still need to handle the indexes
            fold_indexed(val, env, indexes, expr_type)
        }
        Expr::Array(arr) => {
            // Get index
            let ix = indexes.pop().unwrap();

            // Get default initializer if index is out of bounds
            // This makes `int x[N] = {}; x[n]` default initializer instead of poison value
            if ix >= arr.len() {
                return expr_type.default_initializer();
            }

            // Index unfolded array and then fold the result, to save computation
            fold_indexed(&arr[ix], env, indexes, expr_type)
        }
        Expr::Index(arr, ix) => {
            let ix = fold_i32(ix, env)?;
            indexes.push(ix as usize);
            fold_indexed(arr, env, indexes, expr_type)
        }
        _ => Err(anyhow!("expr {:?} can't be indexed", expr)).with_context(|| context!()),
    }
}

/// Fold a potentially array, potentially indexed expression to constant.
pub fn fold_array(expr: &Expr, env: &FrameMap<String, Expr>, element_type: &Type) -> Result<Expr> {
    match expr {
        Expr::Var(id) => {
            let Some(val) = env.get(id) else {
                return Err(anyhow!("Variable not found"));
            };

            // Although val is already folded, we still need to handle the type
            fold_array(val, env, element_type)
        }
        Expr::Array(arr) => arr
            .iter()
            .map(|x| fold_expr(x, env, element_type))
            .collect::<Result<_>>()
            .map(Expr::Array),
        Expr::Index(arr, ix) => {
            let ix = fold_i32(ix, env)?;
            fold_indexed(
                arr,
                env,
                vec![ix as usize],
                &Type::Array(element_type.clone().into(), Expr::Int(0).into()),
            )
        }
        _ => fold_expr(expr, env, element_type),
    }
}

pub fn fold_i32(expr: &Expr, env: &FrameMap<String, Expr>) -> Result<i32> {
    match expr {
        Expr::Var(id) => {
            let Some(val) = env.get(id) else {
                return Err(anyhow!("Variable not found"));
            };

            // Value in environment is already folded, no need to fold again
            val.to_i32()
        }
        Expr::Index(arr, ix) => {
            let ix = fold_i32(ix, env)?;
            fold_indexed(arr, env, vec![ix as usize], &Type::Int)?.to_i32()
        }
        Expr::Int(x) => Ok(*x),
        Expr::Float(x) => Ok(*x as i32),
        Expr::Char(x) => Ok(*x as i32),
        Expr::Bool(x) => Ok(*x as i32),
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
        _ => Err(anyhow!("expr {:?} can't be folded to i32", expr)).with_context(|| context!()),
    }
}

pub fn fold_f32(expr: &Expr, env: &FrameMap<String, Expr>) -> Result<f32> {
    match expr {
        Expr::Var(id) => {
            let Some(val) = env.get(id) else {
                return Err(anyhow!("Variable not found"));
            };
            val.to_f32()
        }
        Expr::Index(arr, ix) => {
            let ix = fold_i32(ix, env)?;
            fold_indexed(arr, env, vec![ix as usize], &Type::Float)?.to_f32()
        }
        Expr::Int(x) => Ok(*x as f32),
        Expr::Float(x) => Ok(*x),
        Expr::Char(x) => Ok(*x as i32 as f32),
        Expr::Bool(x) => Ok(*x as i32 as f32),
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
                    BinaryOp::Shr => return Err(anyhow!("Cannot shift float")),
                    BinaryOp::Shl => return Err(anyhow!("Cannot shift float")),
                    BinaryOp::BitAnd => return Err(anyhow!("Cannot bitwise and float")),
                    BinaryOp::BitOr => return Err(anyhow!("Cannot bitwise or float")),
                    BinaryOp::BitXor => return Err(anyhow!("Cannot bitwise xor float")),
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
        _ => Err(anyhow!("expr {:?} can't be folded to f32", expr)).with_context(|| context!()),
    }
}
