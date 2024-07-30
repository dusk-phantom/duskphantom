use super::*;
use builder::IRBuilder;
use llvm_ir::{constant::Float, Constant, ConstantRef, Type};

use rayon::iter::Either;
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
                        name: _,
                        values,
                        is_packed: _,
                    } => Self::build_array_from_struct(name, &values)?,
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

    pub fn build_array_from_struct(name: &str, values: &[ConstantRef]) -> Result<Var> {
        let (capacity, init) = Self::cap_inits_from_struct(values)?;
        let e_ty = Self::elem_ty_from_struct(values);
        let var: Var = match e_ty {
            Ok(Either::Left(_)) | Err(_) => {
                ArrVar::<u32>::new(name.to_string(), capacity, init, false).into()
            }
            Ok(Either::Right(_)) => {
                let init = init
                    .into_iter()
                    .map(|(i, f)| (i, f32::from_bits(f)))
                    .collect();
                ArrVar::<f32>::new(name.to_string(), capacity, init, false).into()
            }
        };
        Ok(var)
    }
    pub fn cap_inits_from_struct(values: &[ConstantRef]) -> Result<(usize, Vec<(usize, u32)>)> {
        let mut init: Vec<(usize, u32)> = Vec::new();
        let mut capacity = 0;
        for value in values.iter() {
            match value.as_ref() {
                Constant::Int { bits: _, value } => {
                    init.push((capacity, *value as u32));
                    capacity += 1;
                }
                Constant::Float(f) => {
                    if let llvm_ir::constant::Float::Single(f) = f {
                        init.push((capacity, f.to_bits()));
                        capacity += 1;
                    } else {
                        unimplemented!();
                    }
                }
                Constant::AggregateZero(arr) => {
                    let cap = Self::cap_from_empty_array(arr);
                    capacity += cap;
                }
                Constant::Array {
                    element_type,
                    elements,
                } => {
                    let (cap, inits) = Self::cap_inits_from_array(element_type, elements)?;
                    for (idx, v) in inits {
                        let new_idx = idx + capacity;
                        init.push((new_idx, v));
                    }
                    capacity += cap;
                }
                Constant::Struct {
                    name: _,
                    values,
                    is_packed: _,
                } => {
                    let (cap, inits) = Self::cap_inits_from_struct(values)?;
                    for (idx, v) in inits {
                        let new_idx = idx + capacity;
                        init.push((new_idx, v));
                    }
                    capacity += cap;
                }
                _ => {
                    dbg!(value);
                    unimplemented!();
                }
            }
        }
        init.retain(|(_, f)| *f != 0);
        Ok((capacity, init))
    }

    pub fn elem_ty_from_struct(values: &[ConstantRef]) -> Result<Either<i32, f32>> {
        macro_rules! from_e_ty {
            ($e_ty:expr) => {
                if Self::is_ty_int($e_ty) {
                    return Ok(Either::Left(0));
                } else if Self::is_ty_float($e_ty) {
                    return Ok(Either::Right(0.0));
                } else {
                    continue;
                }
            };
        }
        #[allow(clippy::never_loop)]
        for value in values.iter() {
            match value.as_ref() {
                Constant::Int { bits: _, value: _ } => {
                    return Ok(Either::Left(0));
                }
                Constant::Float(_) => {
                    return Ok(Either::Right(0.0));
                }
                Constant::AggregateZero(arr) => {
                    let e_ty = Self::basic_element_type(arr);
                    from_e_ty!(e_ty);
                }
                Constant::Array {
                    element_type,
                    elements: _,
                } => {
                    let e_ty = Self::basic_element_type(element_type);
                    from_e_ty!(e_ty);
                }
                Constant::Struct {
                    name: _,
                    values,
                    is_packed: _,
                } => {
                    let e_ty = Self::elem_ty_from_struct(values);
                    if e_ty.is_ok() {
                        return e_ty;
                    }
                }
                _ => {
                    dbg!(value);
                    unimplemented!();
                }
            }
        }
        Err(anyhow!("no element type found"))
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
                let e_cap = if Self::is_ty_int(element_type) || Self::is_ty_float(element_type) {
                    1
                } else if Self::is_ty_array_type(element_type) {
                    Self::cap_from_empty_array(element_type)
                } else if let llvm_ir::types::Type::StructType {
                    element_types,
                    is_packed: _,
                } = element_type.as_ref()
                {
                    element_types
                        .iter()
                        .map(|t| Self::cap_from_empty_array(t))
                        .sum()
                } else {
                    dbg!(ty);
                    unimplemented!();
                };
                *num_elements * e_cap
            }
            llvm_ir::Type::IntegerType { .. } => 1,
            _ => {
                dbg!(ty);
                unimplemented!();
            }
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
