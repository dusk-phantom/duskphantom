use super::*;
use builder::IRBuilder;
use llvm_ir::{constant::Float, Constant, ConstantRef, Type};

use var::{ArrVar, Var};

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
                    Constant::Array {
                        element_type,
                        elements,
                    } => Self::build_array(name, &element_type, &elements)?,
                    _ => {
                        dbg!(&global_var);
                        unimplemented!()
                    }
                };
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

    /// 处理有初始值的数组
    pub fn build_array(name: &str, e_ty: &Type, elems: &[ConstantRef]) -> Result<Var> {
        dbg!(elems);
        if Self::is_ty_int(e_ty) {
            let mut inits: Vec<(usize, u32)> = Vec::new();
            for (idx, e) in elems.iter().enumerate() {
                let i = Self::build_init_ival(e)?;
                if i == 0 {
                    continue;
                }
                inits.push((idx, i));
            }
            let var = ArrVar::<u32> {
                name: name.to_string(),
                capacity: elems.len(),
                init: inits,
                is_const: false,
            };
            Ok(var.into())
        } else if Self::is_ty_float(e_ty) {
            let mut inits: Vec<(usize, f32)> = Vec::new();
            for (idx, e) in elems.iter().enumerate() {
                let f = Self::build_init_fval(e)?;
                // 如果对应二进制位模式为0，则不需要初始化
                if (f as u32) == 0 {
                    continue;
                }
                inits.push((idx, f));
            }
            let var = ArrVar::<f32> {
                name: name.to_string(),
                capacity: elems.len(),
                init: inits,
                is_const: false,
            };
            Ok(var.into())
        } else {
            // FIXME:  处理嵌套数组
            dbg!(e_ty);
            unimplemented!();
        }
    }
    #[inline]
    pub fn build_init_ival(elem: &Constant) -> Result<u32> {
        match elem {
            Constant::Int { bits: _, value } => Ok(*value as u32),
            _ => unimplemented!(),
        }
    }
    #[inline]
    pub fn build_init_fval(elem: &Constant) -> Result<f32> {
        match elem {
            Constant::Float(f) => match f {
                llvm_ir::constant::Float::Single(f) => Ok(*f),
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }
}
