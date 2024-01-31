use llvm_ir::Module;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs;
use std::process::Command;
use tempfile::NamedTempFile;

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
        cmd.arg("-S")
            .arg("-emit-llvm")
            .arg("-Xclang")
            .arg("-disable-O0-optnone")
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
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n", self.llvm.name).unwrap();
        write!(f, "global_vars:\n",).unwrap();
        for global_var in &self.llvm.global_vars {
            write!(f, "{}\n", &global_var.name.to_string()[1..]).unwrap();
        }
        write!(f, "functions:\n",).unwrap();
        for func in &self.llvm.functions {
            write!(f, "{}", func.name).unwrap();
        }
        Ok(())
    }
}

pub fn optimize(program: &mut Program) {
    // 使用clang 命令优化.ll 代码
    let llvm_path = program.tmp_llvm_file.path();
    let mut cmd = Command::new("clang");
    cmd.arg("-S")
        .arg("-O2")
        .arg("-emit-llvm")
        .arg(llvm_path)
        .arg("-o")
        .arg(llvm_path);
    let output = cmd.output().expect("msg: exec clang failed");
    if !output.status.success() {
        panic!("msg: exec clang failed");
    }
    // 使用llvm_ir crate从.ll内容文件中读取llvm ir
    let llvm = Module::from_ir_path(llvm_path).expect("msg: parse llvm ir failed");
    program.llvm = llvm;
}
