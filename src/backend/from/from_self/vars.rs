use super::builder::IRBuilder;
use irs::var::ArrVar;
use middle::ir::Constant;

use crate::backend::var::{self, Var};

use anyhow::Result;

use crate::backend::*;
use crate::middle;

impl IRBuilder {
    pub fn build_global_var(self_global_vars: &Vec<middle::ir::GlobalPtr>) -> Result<Vec<Var>> {
        let mut global_vars = Vec::new();
        for global_var in self_global_vars {
            // dbg!(&global_var);
            let name = &global_var.name;
            let new_var = match &global_var.initializer {
                middle::ir::Constant::SignedChar(_) => todo!(),
                middle::ir::Constant::Int(i) => Self::build_int_var(name, *i)?,
                middle::ir::Constant::Float(f) => Self::build_float_var(name, *f)?,
                middle::ir::Constant::Bool(b) => Self::build_bool_var(name, *b)?,
                middle::ir::Constant::Array(arr) => Self::build_arr_var(name, arr)?,
            };
            global_vars.push(new_var);
        }
        Ok(global_vars)
    }

    #[allow(unused)]
    fn build_arr_var(name: &str, arr: &Vec<Constant>) -> Result<Var> {
        let flattened = Self::_flatten_arr(arr)?;
        if let Some(first) = flattened.first() {
            match first {
                Constant::Int(_) => {
                    let mut arr: Vec<(usize, u32)> = Vec::new();
                    for (index, item) in flattened.iter().enumerate() {
                        let Constant::Int(i) = item else {
                            return Err(anyhow!("can't not reveive a mixed arr: {:?}", first))
                                .with_context(|| context!());
                        };
                        arr.push((index, *i as u32)); // 这个应该是位模式的转换
                    }
                    let var = ArrVar {
                        name: name.to_string(),
                        capacity: arr.len(),
                        init: arr,
                        is_const: false,
                    };
                    Ok(var.into())
                }
                Constant::Float(_) => {
                    let mut arr: Vec<(usize, f32)> = Vec::new();
                    for (index, item) in flattened.iter().enumerate() {
                        let Constant::Float(f) = item else {
                            return Err(anyhow!("can't not reveive a mixed arr: {:?}", first))
                                .with_context(|| context!());
                        };
                        arr.push((index, *f)); // 这个应该是位模式的转换
                    }
                    let var = ArrVar {
                        name: name.to_string(),
                        capacity: arr.len(),
                        init: arr,
                        is_const: false,
                    };
                    Ok(var.into())
                }
                Constant::SignedChar(_) => unimplemented!(),
                Constant::Bool(_) => unimplemented!(),
                Constant::Array(_) => {
                    Err(anyhow!("arr has been flattened: {:?}", first)).with_context(|| context!())
                } // Cons
            }
        } else {
            Err(anyhow!("backend get an empty array from middle: {:?}", arr))
                .with_context(|| context!())
        }
    }

    /// 递归的展平数组
    fn _flatten_arr(arr: &Vec<Constant>) -> Result<Vec<Constant>> {
        let mut flattened = Vec::new();
        for item in arr {
            match item {
                Constant::Array(sub_arr) => {
                    let sub_flattened = Self::_flatten_arr(sub_arr)?;
                    flattened.extend(sub_flattened);
                }
                _ => flattened.push(item.clone()),
            }
        }
        Ok(flattened)
    }

    fn build_int_var(name: &str, value: i32) -> Result<Var> {
        let var = var::Var::Prim(var::PrimVar::IntVar(var::IntVar {
            name: name.to_string(),
            init: Some(value),
            is_const: false,
        }));
        Ok(var)
    }

    fn build_bool_var(name: &str, value: bool) -> Result<Var> {
        let var = var::Var::Prim(var::PrimVar::IntVar(var::IntVar {
            name: name.to_string(),
            init: Some(value as i32),
            is_const: false,
        }));
        Ok(var)
    }

    pub fn build_float_var(name: &str, f: f32) -> Result<Var> {
        let var = var::Var::Prim(var::PrimVar::FloatVar(var::FloatVar {
            name: name.to_string(),
            init: Some(f),
            is_const: false,
        }));
        Ok(var)
    }
}
