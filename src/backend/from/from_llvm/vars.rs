use super::*;
use builder::IRBuilder;
use llvm_ir::{constant::Float, Constant, Type};

use var::Var;

impl IRBuilder {
    pub fn build_global_var(
        llvm_global_vars: &[llvm_ir::module::GlobalVariable],
    ) -> Result<Vec<Var>> {
        let mut global_vars = Vec::new();
        for global_var in llvm_global_vars {
            // dbg!(&global_var);
            let name = &global_var.name.to_string()[1..];
            if let Some(init) = &global_var.initializer {
                let c = init.as_ref().to_owned();
                let new_var = match c {
                    Constant::Int { bits: _, value } => Self::build_int_var(name, value)?,
                    Constant::Float(f) => Self::build_float_var(name, f)?,
                    // 处理0初始化的数组
                    Constant::AggregateZero(arr) => Self::build_aggregate_zero_var(name, &arr)?,
                    _ => {
                        dbg!(&global_var);
                        unimplemented!()
                    }
                };
                dbg!(&new_var);
                global_vars.push(new_var);
            }
        }
        Ok(global_vars)
    }

    pub fn build_int_var(name: &str, value: u64) -> Result<Var> {
        let var = var::Var::Prim(var::PrimVar::IntVar(var::IntVar {
            name: name.to_string(),
            init: Some(value as i32),
            is_const: false,
        }));
        Ok(var)
    }

    pub fn build_float_var(name: &str, f: Float) -> Result<Var> {
        match f {
            llvm_ir::constant::Float::Single(f) => {
                let var = var::Var::Prim(var::PrimVar::FloatVar(var::FloatVar {
                    name: name.to_string(),
                    init: Some(f),
                    is_const: false,
                }));
                Ok(var)
            }
            llvm_ir::constant::Float::Double(_) => {
                unimplemented!("double float");
                // let var = var::Var::Prim(var::PrimVar::FloatVar(var::FloatVar {
                //     name: name.to_string(),
                //     init: Some(f),
                //     is_const: false,
                // }));
                // global_vars.push(var);
            }
            _ => {
                dbg!(f);
                unreachable!();
            }
        }
    }

    /// 处理0初始化的数组
    pub fn build_aggregate_zero_var(name: &str, arr: &Type) -> Result<Var> {
        match arr {
            llvm_ir::Type::ArrayType {
                element_type,
                num_elements,
            } => {
                if Self::is_ty_int(element_type) {
                    let var = irs::var::ArrVar::<u32> {
                        name: name.to_string(),
                        capacity: *num_elements,
                        init: vec![],
                        is_const: false,
                    };
                    Ok(var.into())
                } else if Self::is_ty_float(element_type) {
                    let var = irs::var::ArrVar::<f32> {
                        name: name.to_string(),
                        capacity: *num_elements,
                        init: vec![],
                        is_const: false,
                    };
                    Ok(var.into())
                } else {
                    unimplemented!();
                }
            }
            _ => unimplemented!(),
        }
    }
}
