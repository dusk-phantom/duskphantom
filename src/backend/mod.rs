pub mod block;
pub mod func;
pub mod gen_asm;
pub mod inst;
pub mod module;
pub mod prog;
pub mod var;

use std::any::{Any, TypeId};

use llvm_ir::types::Typed;
use rayon::prelude::*;

use crate::{errors::BackendError, middle};

#[cfg(feature = "clang_frontend")]
use crate::clang_frontend;

#[allow(unused)]
pub fn gen(program: &middle::Program) -> Result<prog::Program, BackendError> {
    // TODO
    Ok(prog::Program {
        entry: None,
        modules: vec![],
    })
}

#[cfg(feature = "clang_frontend")]
#[allow(unused)]
pub fn gen_from_clang(program: &clang_frontend::Program) -> Result<prog::Program, BackendError> {
    let mut global_vas = Vec::new();
    let mut funcs = Vec::new();
    for global_var in &program.llvm.global_vars {
        let name = &global_var.name.to_string()[1..];
        let ty = &global_var.ty.to_string();
        // let ty= match ty.as_ref() {
        //     "i32" => var::VarType::Int,
        // }
        println!("{} {}", name, ty);
        // let ty = &global_var.type_id() {
        //     // TypeId::of<String>() => var::VarType::String,
        //     // _=>(),
        // };

        // let var = var::Var {
        //     name: global_var.name.to_string(),
        //     ty: var::VarType::Int,
        //     is_const: false,
        //     is_global: true,
        //     is_array: false,
        //     array_size: 0,
        //     init_val: None,
        // };
        // global_vas.push(var);
    }

    let mdl = module::Module {
        name: "main".to_string(),
        entry: Some("main".to_string()),
        global: global_vas,
        funcs,
    };
    Ok(prog::Program {
        entry: None,
        modules: vec![mdl],
    })
}

#[allow(unused)]
pub fn optimize(program: &mut prog::Program) {
    // 乘除法优化
    // 块重排
    // 指令调度
    // 寄存器分配与合并
}

#[allow(unused)]
pub fn asm2bin(asm: String) -> String {
    panic!("not implemented")
}
