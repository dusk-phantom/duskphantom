use super::*;
use std::process::Command;

// 从clang frontend 生成riscv 汇编
pub struct Program {
    pub asm:String
}

pub fn optimize(program: &mut Program) {
    // TODO
}


pub fn gen_from_clang(program: &clang_frontend::Program) -> Result<Program, CompilerError> {
    let tmp_llvm_file = program.tmp_llvm_file.path();
    let mut cmd = Command::new("llc");
    cmd.arg("-march=riscv64")
        .arg("-mattr=+m,+f,+d,+a,+c")
        .arg(tmp_llvm_file)
        .arg("-o")
        .arg(tmp_llvm_file);
    let output = cmd.output().expect("msg: exec llc failed");
    if !output.status.success() {
        panic!("msg: exec llc failed");
    }
    let asm = std::fs::read_to_string(tmp_llvm_file).expect("msg: read asm failed");
    Ok(Program { asm })
}

impl Program {
    pub fn gen_asm(&mut self) -> String {
        self.asm.clone()
    }
    
}