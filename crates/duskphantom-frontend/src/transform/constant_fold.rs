// Copyright 2024 Duskphantom Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use std::collections::VecDeque;

use anyhow::{anyhow, Context, Result};

use crate::{BinaryOp, Decl, Expr, Program, Stmt, Type, TypedIdent, UnaryOp};
use duskphantom_utils::context;
use duskphantom_utils::frame_map::FrameMap;

use super::reshape_array::{reshape_array, reshape_const_array};

pub fn optimize_program(program: &mut Program) -> Result<()> {
    let mut env = FrameMap::new();
    for decl in program.module.iter_mut() {
        fold_decl(decl, &mut env, true)?;
    }
    Ok(())
}

/// Fold constant expression in declaration into constant.
fn fold_decl(decl: &mut Decl, env: &mut FrameMap<String, Expr>, is_global: bool) -> Result<()> {
    match decl {
        Decl::Const(ty, id, expr) => {
            // Fold type
            *ty = get_folded_type(ty, env)?;

            // Calculate folded initializer
            let mut folded: Expr;
            match expr {
                Some(expr) => {
                    // Calculate from given initializer
                    folded = get_folded_expr(expr, env, ty)?;

                    // Constant array can be malformed, reshape it
                    if let Expr::Array(arr) = folded {
                        folded = reshape_const_array(&mut VecDeque::from(arr), ty)?;
                    }
                }
                None => {
                    // Use default initializer
                    folded = ty.default_initializer()?;
                }
            }

            // Update expression to folded
            *expr = Some(folded.clone());

            // Insert folded expression to environment
            env.insert(id.clone(), folded);
        }
        Decl::Var(ty, _, expr) => {
            // Fold type
            *ty = get_folded_type(ty, env)?;

            // If variable is global, initializer should be constant
            if is_global {
                // Calculate folded initializer
                let mut folded: Expr;
                match expr {
                    Some(expr) => {
                        // Calculate from given initializer
                        folded = get_folded_expr(expr, env, ty)?;

                        // Constant array can be malformed, reshape it
                        if let Expr::Array(arr) = folded {
                            folded = reshape_const_array(&mut VecDeque::from(arr), ty)?;
                        }
                    }
                    None => {
                        // Use default initializer
                        folded = ty.default_initializer()?;
                    }
                }

                // Update expression to folded
                *expr = Some(folded.clone());
            } else {
                // Value array can be malformed, reshape it
                if let Some(Expr::Array(arr)) = expr {
                    *expr = Some(reshape_array(&mut VecDeque::from(arr.clone()), ty)?);
                }
            }
        }
        Decl::Stack(vec) => {
            for decl in vec {
                fold_decl(decl, env, is_global)?;
            }
        }
        Decl::Func(ty, _, Some(stmt)) => {
            *ty = get_folded_type(ty, env)?;
            fold_stmt(stmt, &mut env.branch())?;
        }
        _ => (),
    }
    Ok(())
}

/// Fold constant expression in statement into constant.
fn fold_stmt(stmt: &mut Stmt, env: &mut FrameMap<String, Expr>) -> Result<()> {
    match stmt {
        Stmt::Decl(decl) => fold_decl(decl, env, false)?,
        Stmt::Block(vec) => {
            let mut inner_env = env.branch();
            for stmt in vec {
                fold_stmt(stmt, &mut inner_env)?;
            }
        }
        Stmt::If(_, a, b) => {
            fold_stmt(a, env)?;
            fold_stmt(b, env)?;
        }
        Stmt::While(_, a) => fold_stmt(a, env)?,
        Stmt::DoWhile(a, _) => fold_stmt(a, env)?,
        _ => (),
    }
    Ok(())
}

impl Expr {
    pub fn to_i32(&self) -> Result<i32> {
        match self {
            Expr::Int(x) => Ok(*x),
            Expr::Float(x) => Ok(*x as i32),
            _ => Err(anyhow!("Cannot cast to i32")),
        }
    }

    pub fn to_f32(&self) -> Result<f32> {
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

/// Fold a type to constant.
fn get_folded_type(ty: &Type, env: &FrameMap<String, Expr>) -> Result<Type> {
    match ty {
        Type::Pointer(ty) => Ok(Type::Pointer(get_folded_type(ty, env)?.into())),
        Type::Array(element_type, size) => {
            let size = get_folded_i32(size, env)?;
            let element_type = get_folded_type(element_type, env)?;
            Ok(Type::Array(element_type.into(), Expr::Int(size).into()))
        }
        Type::Function(ret, params) => {
            // Fold return type
            let ret = get_folded_type(ret, env)?;

            // For each param, fold the type it contains
            let params = params
                .iter()
                .map(|ti| get_folded_type(&ti.ty, env).map(|ty| TypedIdent::new(ty, ti.id.clone())))
                .collect::<Result<_>>()?;

            // Reconstruct function type
            Ok(Type::Function(ret.into(), params))
        }
        _ => Ok(ty.clone()),
    }
}

/// Fold an expression to constant.
fn get_folded_expr(expr: &Expr, env: &FrameMap<String, Expr>, expr_type: &Type) -> Result<Expr> {
    match expr_type {
        Type::Array(element_type, _) => {
            let arr = get_folded_array(expr, env, element_type)?;
            Ok(arr)
        }
        Type::Int => {
            let x = get_folded_i32(expr, env)?;
            Ok(Expr::Int(x))
        }
        Type::Float => {
            let x = get_folded_f32(expr, env)?;
            Ok(Expr::Float(x))
        }
        _ => Err(anyhow!("cannot fold an instance of {:?}", expr_type)).with_context(|| context!()),
    }
}

/// Fold an indexed expression to constant.
fn get_folded_indexed(
    expr: &Expr,
    env: &FrameMap<String, Expr>,
    mut indexes: Vec<usize>,
    expr_type: &Type,
) -> Result<Expr> {
    // If not indexed, fallback to regular expression fold
    if indexes.is_empty() {
        return get_folded_expr(expr, env, expr_type);
    }

    // Expression is indexed
    match expr {
        Expr::Var(id) => {
            let Some(val) = env.get(id) else {
                return Err(anyhow!("Variable not found"));
            };

            // Although val is already folded, we still need to handle the indexes
            get_folded_indexed(val, env, indexes, expr_type)
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
            get_folded_indexed(&arr[ix], env, indexes, expr_type)
        }
        Expr::Index(arr, ix) => {
            let ix = get_folded_i32(ix, env)?;
            indexes.push(ix as usize);
            get_folded_indexed(arr, env, indexes, expr_type)
        }
        _ => Err(anyhow!("expr {:?} can't be indexed", expr)).with_context(|| context!()),
    }
}

/// Fold an array to constant.
fn get_folded_array(
    expr: &Expr,
    env: &FrameMap<String, Expr>,
    element_type: &Type,
) -> Result<Expr> {
    match expr {
        Expr::Var(id) => {
            let Some(val) = env.get(id) else {
                return Err(anyhow!("Variable not found"));
            };

            // Although val is already folded, we still need to handle the type
            get_folded_array(val, env, element_type)
        }
        Expr::Array(arr) => arr
            .iter()
            .map(|x| get_folded_expr(x, env, element_type))
            .collect::<Result<_>>()
            .map(Expr::Array),
        Expr::Index(arr, ix) => {
            let ix = get_folded_i32(ix, env)?;
            get_folded_indexed(
                arr,
                env,
                vec![ix as usize],
                &Type::Array(element_type.clone().into(), Expr::Int(0).into()),
            )
        }
        _ => get_folded_expr(expr, env, element_type),
    }
}

/// Fold an i32 to constant.
fn get_folded_i32(expr: &Expr, env: &FrameMap<String, Expr>) -> Result<i32> {
    match expr {
        Expr::Var(id) => {
            let Some(val) = env.get(id) else {
                return Err(anyhow!("Variable not found"));
            };

            // Value in environment is already folded, no need to fold again
            val.to_i32()
        }
        Expr::Index(arr, ix) => {
            let ix = get_folded_i32(ix, env)?;
            get_folded_indexed(arr, env, vec![ix as usize], &Type::Int)?.to_i32()
        }
        Expr::Int(x) => Ok(*x),
        Expr::Float(x) => Ok(*x as i32),
        Expr::Bool(x) => Ok(*x as i32),
        Expr::Unary(op, expr) => {
            let x = get_folded_i32(expr, env)?;
            match op {
                UnaryOp::Neg => Ok(-x),
                UnaryOp::Pos => Ok(x),
                UnaryOp::Not => Ok(if x == 0 { 1 } else { 0 }),
            }
        }
        Expr::Binary(head, tail) => {
            let mut x = get_folded_i32(head, env)?;
            for (op, expr) in tail {
                let y = get_folded_i32(expr, env)?;
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

/// Fold an f32 to constant.
fn get_folded_f32(expr: &Expr, env: &FrameMap<String, Expr>) -> Result<f32> {
    match expr {
        Expr::Var(id) => {
            let Some(val) = env.get(id) else {
                return Err(anyhow!("Variable not found"));
            };
            val.to_f32()
        }
        Expr::Index(arr, ix) => {
            let ix = get_folded_i32(ix, env)?;
            get_folded_indexed(arr, env, vec![ix as usize], &Type::Float)?.to_f32()
        }
        Expr::Int(x) => Ok(*x as f32),
        Expr::Float(x) => Ok(*x),
        Expr::Bool(x) => Ok(*x as i32 as f32),
        Expr::Unary(op, expr) => {
            let x = get_folded_f32(expr, env)?;
            match op {
                UnaryOp::Neg => Ok(-x),
                UnaryOp::Pos => Ok(x),
                UnaryOp::Not => Ok(if x == 0.0 { 1.0 } else { 0.0 }),
            }
        }
        Expr::Binary(head, tail) => {
            let mut x = get_folded_f32(head, env)?;
            for (op, expr) in tail {
                let y = get_folded_f32(expr, env)?;
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
