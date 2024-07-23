use middle::ir::Constant;

use super::*;

use crate::backend::var::{self, Var};

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
    fn build_arr_var(name: &str, value: &Vec<Constant>) -> Result<Var> {
        let _ = value;
        todo!()
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
