use tempfile::NamedTempFile;

use super::*;
use std::process::Command;

// 从clang frontend 生成riscv 汇编
pub struct Program {
    pub tmp_llvm_file: NamedTempFile,
    pub opt: bool,
}

#[allow(unused)]
pub fn optimize(program: &mut Program) {
    program.opt = true;
}

pub fn gen_from_clang(program: &clang_frontend::Program) -> Result<Program, CompilerError> {
    let tmp_llvm_file = NamedTempFile::new().expect("msg: create tmp_llvm_file failed");
    let mut cmd = Command::new("cp");
    cmd.arg(program.tmp_llvm_file.path())
        .arg(tmp_llvm_file.path());
    let output = cmd.output().expect("msg: exec clang failed");
    if !output.status.success() {
        panic!("msg: exec clang failed");
    }
    Ok(Program {
        tmp_llvm_file,
        opt: false,
    })
}

impl Program {
    pub fn gen_asm(&mut self) -> String {
        let tmp_llvm_file = self.tmp_llvm_file.path();
        let mut cmd = Command::new("llc");
        cmd.arg("-march=riscv64")
            .arg("-mattr=+m,+f,+d,+a,+c")
            .arg(tmp_llvm_file)
            .arg("-o")
            .arg(tmp_llvm_file);
        if self.opt {
            cmd.arg("-O3");
        }
        let output = cmd.output().expect("msg: exec llc failed");
        if !output.status.success() {
            panic!(
                "msg: exec llc failed\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::fs::read_to_string(tmp_llvm_file).expect("msg: read asm failed")
    }
}
