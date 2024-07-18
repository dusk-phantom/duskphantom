use anyhow::anyhow;
use anyhow::Context;
use llvm_ir::Module;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs;
use std::process::Command;
use tempfile::NamedTempFile;

use crate::context;

pub struct Program {
    pub tmp_cfile: NamedTempFile,
    pub tmp_llvm_file: NamedTempFile,
    pub llvm: Module,
}

impl Program {
    pub fn parse(file: &str) -> Self {
        let mut builder = tempfile::Builder::new();
        let tmp_cfile = builder.suffix(".c").tempfile().unwrap();
        let tmp_llvm_file = builder.suffix(".ll").tempfile().unwrap();
        fs::copy(file, tmp_cfile.path()).expect("msg: copy file failed");
        let mut cmd = Command::new("clang");
        // clang -S -emit-llvm -Xclang -disable-O0-optnone -target riscv64 1.c -o 1.ll
        cmd.arg("-S")
            .arg("-emit-llvm")
            .arg("-Wno-implicit-function-declaration")
            .arg("-Werror")
            .arg("-Xclang")
            .arg("-disable-O0-optnone")
            .arg("-target")
            .arg("riscv64")
            // 制定使用非.c后缀的文件名
            .arg(tmp_cfile.path())
            .arg("-o")
            .arg(tmp_llvm_file.path());
        let output = cmd.output().expect("msg: exec clang failed");

        if !output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stderr));
            println!("{}", String::from_utf8_lossy(&output.stdout));
            panic!("msg: exec clang failed");
        }
        // 使用llvm_ir crate从.ll内容文件中读取llvm ir
        let llvm = Module::from_ir_path(tmp_llvm_file.path()).expect("msg: parse llvm ir failed");
        Self {
            tmp_cfile,
            tmp_llvm_file,
            llvm,
        }
    }
    pub fn gen_ll(&self) -> anyhow::Result<String> {
        fs::read_to_string(self.tmp_llvm_file.path())
            .map_err(|e| anyhow!("{e}"))
            .with_context(|| context!())
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        s.push_str(&self.llvm.name);
        s.push('\n');
        s.push_str("global_vars:\n");
        for global_var in &self.llvm.global_vars {
            s.push_str(&global_var.name.to_string()[1..]);
            s.push('\n');
        }
        s.push_str("functions:\n");
        for func in &self.llvm.functions {
            s.push_str(&func.name);
        }
        write!(f, "{}", s)?;
        Ok(())
    }
}

pub fn optimize(program: &mut Program) -> anyhow::Result<()> {
    // 使用clang 命令优化.ll 代码
    let llvm_path = program.tmp_llvm_file.path();
    let mut cmd = Command::new("opt");
    cmd.arg("-S")
        .arg("-O3")
        .arg(llvm_path)
        .arg("-o")
        .arg(llvm_path);
    let output = cmd
        .output()
        .map_err(|e| anyhow!("{e}"))
        .with_context(|| context!())?;

    if !output.status.success() {
        return Err(anyhow!("msg: exec opt failed")).with_context(|| context!());
    }

    let llvm = Module::from_ir_path(llvm_path)
        .map_err(|e| anyhow!("{e}"))
        .with_context(|| context!())?;
    program.llvm = llvm;
    Ok(())
}
