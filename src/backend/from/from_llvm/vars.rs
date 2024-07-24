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
            // continue;
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
                    } => Self::build_array_var(name, &element_type, &elements)?,
                    Constant::Struct {
                        name,
                        values,
                        is_packed,
                    } => {
                        dbg!(name, values, is_packed);
                        unimplemented!();
                    }
                    _ => {
                        // dbg!(&global_var);
                        unimplemented!()
                    }
                };
                // dbg!(&new_var);
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
            }
            _ => {
                dbg!(f);
                unreachable!();
            }
        }
    }

    /// 处理0初始化的数组
    #[inline]
    pub fn build_aggregate_zero_var(name: &str, arr: &Type) -> Result<Var> {
        let capacity = Self::cap_from_empty_array(arr);
        let base_ty = Self::basic_element_type(arr);
        if Self::is_ty_int(base_ty) {
            let var = ArrVar::<u32> {
                name: name.to_string(),
                init: vec![],
                capacity,
                is_const: false,
            };
            Ok(var.into())
        } else if Self::is_ty_float(base_ty) {
            let var = ArrVar::<f32> {
                name: name.to_string(),
                init: vec![],
                capacity,
                is_const: false,
            };
            Ok(var.into())
        } else {
            dbg!(base_ty);
            unimplemented!();
        }
    }

    #[allow(clippy::if_same_then_else)]
    pub fn cap_from_empty_array(ty: &llvm_ir::Type) -> usize {
        match ty {
            llvm_ir::Type::ArrayType {
                element_type,
                num_elements,
            } => {
                if Self::is_ty_int(element_type) {
                    *num_elements
                } else if Self::is_ty_float(element_type) {
                    *num_elements
                } else if Self::is_ty_array_type(element_type) {
                    let sub = Self::cap_from_empty_array(element_type);
                    *num_elements * sub
                } else {
                    dbg!(ty);
                    unimplemented!();
                }
            }
            _ => unimplemented!(),
        }
    }

    pub fn is_ty_array_type(ty: &llvm_ir::Type) -> bool {
        matches!(
            ty,
            Type::ArrayType {
                element_type: _,
                num_elements: _
            }
        )
    }

    #[inline]
    pub fn build_array_var(
        name: &str,
        element_type: &Type,
        elements: &[ConstantRef],
    ) -> Result<Var> {
        let base_ty = Self::basic_element_type(element_type);
        let (capacity, init) = Self::cap_inits_from_array(element_type, elements)?;
        let new_var = if Self::is_ty_int(base_ty) {
            ArrVar::<u32>::new(name.to_string(), capacity, init, false).into()
        } else if Self::is_ty_float(base_ty) {
            let init = init
                .into_iter()
                .map(|(i, f)| (i, f32::from_bits(f)))
                .collect();
            dbg!(&init);
            ArrVar::<f32>::new(name.to_string(), capacity, init, false).into()
        } else {
            dbg!(base_ty);
            unimplemented!();
        };
        Ok(new_var)
    }

    /// 处理有初始值的数组,返回(数组容量,数组初始值)
    #[allow(unused)]
    pub fn cap_inits_from_array(
        e_ty: &Type,
        elems: &[ConstantRef],
    ) -> Result<(usize, Vec<(usize, u32)>)> {
        // 首先处理递归基线 process recursive base line
        // 递归基线1: 空数组 base line 1: empty array
        if elems.is_empty() {
            let capacity = Self::cap_from_empty_array(e_ty);
            return Ok((capacity, vec![]));
        }
        // 递归基线2: 数组元素为常量 base line 2: array elements are constants
        if Self::is_ty_float(e_ty) || Self::is_ty_int(e_ty) {
            let mut init = Vec::new();
            for (i, elem) in elems.iter().enumerate() {
                if let Constant::Float(f) = elem.as_ref() {
                    if let llvm_ir::constant::Float::Single(f) = f {
                        init.push((i, f.to_bits()));
                    } else {
                        unimplemented!();
                    }
                } else if let Constant::Int { bits: _, value } = elem.as_ref() {
                    init.push((i, *value as u32));
                } else {
                    unimplemented!();
                }
            }
            init.retain(|(_, f)| *f != 0);
            return Ok((elems.len(), init));
        }
        // 递归部分 recursive part
        let mut total_capacity: usize = 0;
        let mut total_inits = vec![];
        for elem in elems {
            if let Constant::Array {
                element_type,
                elements,
            } = elem.as_ref()
            {
                let (cap, inits) = Self::cap_inits_from_array(element_type, elements)?;
                total_inits.extend(inits.into_iter().map(|(i, f)| (i + total_capacity, f)));
                total_capacity += cap;
            } else if let Constant::AggregateZero(arr) = elem.as_ref() {
                let cap = Self::cap_from_empty_array(arr);
                total_capacity += cap;
            } else {
                unimplemented!();
            }
        }
        // dbg!(e_ty, elems);
        // dbg!(total_capacity, &total_inits);
        Ok((total_capacity, total_inits))
    }

    #[inline]
    pub fn basic_element_type(ty: &llvm_ir::Type) -> &llvm_ir::Type {
        match ty {
            llvm_ir::Type::ArrayType {
                element_type,
                num_elements: _,
            } => Self::basic_element_type(element_type),
            llvm_ir::Type::IntegerType { .. } => ty,
            llvm_ir::Type::FPType { .. } => ty,
            _ => unimplemented!("basic_element_type"),
        }
    }
}
