use var::ArrVar;

use super::*;

use crate::errors::*;
use crate::middle;
use crate::middle::ir::Constant;

#[allow(unused)]
pub fn gen(program: &middle::Program) -> Result<prog::Program, BackendError> {
    let mut global_vars = Vec::new();
    let mut funcs = Vec::new();
    let llvm = &program.module;
    // dbg!(&llvm.types);
    for global_var in &llvm.global_variables {
        // dbg!(&global_var);
        let name = &global_var.name.to_string()[1..]; // FIXME 是不是这样 ？ 有待验证
        if let c = &global_var.initializer {
            match c {
                Constant::Int(value) => {
                    let var = var::Var::Prim(var::PrimVar::IntVar(var::IntVar {
                        name: name.to_string(),
                        init: Some(*value as i32),
                        is_const: false,
                    }));
                    global_vars.push(var);
                }
                Constant::Float(value) => {
                    let var = var::Var::Prim(var::PrimVar::FloatVar(var::FloatVar {
                        name: name.to_string(),
                        init: Some(*value as f32),
                        is_const: false,
                    }));
                    global_vars.push(var);
                }
                Constant::Bool(value) => {
                    let var = var::Var::Prim(var::PrimVar::IntVar(var::IntVar {
                        name: name.to_string(),
                        init: Some(*value as i32),
                        is_const: false,
                    }));
                    global_vars.push(var);
                }
                Constant::Array(arr) => {
                    match arr.first().unwrap() /* FIXME unwrap */ {
                        Constant::Int(_) => {
                            let mut init = Vec::new();
                            for (index, con) in arr.iter().enumerate() {
                                if let Constant::Int(value) = con {
                                    init.push((index, *value as i32));
                                } else {
                                    unreachable!();
                                }
                            }
                            let arr_var = var::Var::IntArr(ArrVar::<i32> {
                                name: name.to_string(),
                                capacity: arr.len(),
                                init,
                                is_const: false /* TODO */,
                            });
                            global_vars.push(arr_var);
                        }
                        Constant::Float(_) => {
                            let mut init = Vec::new();
                            for (index, con) in arr.iter().enumerate() {
                                if let Constant::Float(value) = con {
                                    init.push((index, *value as f32));
                                } else {
                                    unreachable!();
                                }
                            }
                            let arr_var = var::Var::FloatArr(ArrVar::<f32> {
                                name: name.to_string(),
                                capacity: arr.len(),
                                init,
                                is_const: false /* TODO */,
                            });
                            global_vars.push(arr_var);
                        }
                        Constant::Bool(_) => {
                            let mut init = Vec::new();
                            for (index, con) in arr.iter().enumerate() {
                                if let Constant::Bool(value) = con {
                                    init.push((index, *value as i32));
                                } else {
                                    unreachable!();
                                }
                            }
                            let arr_var = var::Var::IntArr(ArrVar::<i32> {
                                name: name.to_string(),
                                capacity: arr.len(),
                                init,
                                is_const: false /* TODO */,
                            });
                            global_vars.push(arr_var);
                        }
                        _ => {
                            /* FIXME */ unreachable!();
                        }
                    }
                }
            }
        }
    }
    // dbg!(&global_vars);

    // TODO
    Ok(prog::Program {
        entry: None,
        modules: vec![],
    })
}
