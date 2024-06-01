use super::*;

use super::super::prog::Program;
use crate::clang_frontend;
use crate::errors::BackendError;

#[cfg(feature = "clang_embeded")]
#[allow(unused)]
pub fn gen_from_clang(program: &clang_frontend::Program) -> Result<Program, BackendError> {
    use llvm_ir::Constant;

    let mut global_vars = Vec::new();
    let mut funcs = Vec::new();
    let llvm = &program.llvm;
    for global_var in &llvm.global_vars {
        let name = &global_var.name.to_string()[1..];
        if let Some(init) = &global_var.initializer {
            let c = init.as_ref().to_owned();
            match c {
                Constant::Int { bits, value } => {
                    let var = var::Var::Prim(var::PrimVar::IntVar(var::IntVar {
                        name: name.to_string(),
                        init: Some(value as i64),
                        is_const: false,
                    }));
                    global_vars.push(var);
                }
                Constant::Float(f) => match f {
                    llvm_ir::constant::Float::Single(f) => {
                        let var = var::Var::Prim(var::PrimVar::FloatVar(var::FloatVar {
                            name: name.to_string(),
                            init: Some(f as f64),
                            is_const: false,
                        }));
                        global_vars.push(var);
                    }
                    llvm_ir::constant::Float::Double(f) => {
                        let var = var::Var::Prim(var::PrimVar::FloatVar(var::FloatVar {
                            name: name.to_string(),
                            init: Some(f),
                            is_const: false,
                        }));
                        global_vars.push(var);
                    }
                    _ => {
                        unreachable!();
                    }
                },
                _ => (),
            }
        }
    }
    dbg!(&global_vars);
    // for f in &llvm.functions {
    //     dbg!(f);
    // }
    let mdl = module::Module {
        name: "main".to_string(),
        entry: Some("main".to_string()),
        global: global_vars,
        funcs,
    };
    Ok(prog::Program {
        entry: None,
        modules: vec![mdl],
    })
}
