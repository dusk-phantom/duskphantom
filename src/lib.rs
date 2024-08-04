use anyhow::Context;
use errors::CompilerError;
use std::fs;
use std::os::unix::fs::PermissionsExt;
pub mod args;
pub mod backend;

#[cfg(feature = "clang_enabled")]
pub mod clang_backend;
#[cfg(feature = "clang_enabled")]
pub mod clang_frontend;
pub mod config;
pub mod errors;
pub mod frontend;
pub mod middle;
pub mod utils;
#[cfg(not(feature = "gen_virtual_asm"))]
use backend::checker;

use clap::{arg, App};

/// compile sysy source code to rv64gc asm
pub fn compile(
    sy_path: &str,
    output_path: &str,
    opt_flag: bool,
    asm_flag: bool,
    ll_path: Option<String>,
) -> Result<(), CompilerError> {
    let content = std::fs::read_to_string(sy_path).map_err(CompilerError::IOError)?;
    let mut program = frontend::parse(&content)?;
    if opt_flag {
        frontend::optimize(&mut program);
    }
    let mut program = middle::gen(&program)?;
    if opt_flag {
        middle::optimize(&mut program);
    }
    if let Some(ll_path) = ll_path {
        std::fs::write(ll_path, program.module.gen_llvm_ir()).with_context(|| context!())?;
    }
    let mut program = backend::from_self::gen_from_self(&program)?;

    if opt_flag {
        backend::optimize(&mut program)?;
    } else {
        backend::phisicalize(&mut program)?;
    }

    // check valid
    #[cfg(not(feature = "gen_virtual_asm"))]
    checker::ProgramChecker::check_valid(&checker::Riscv, &program);

    let asm = program.gen_asm();
    output(asm, output_path, asm_flag)
}

#[cfg(feature = "clang_enabled")]
/// compile from clang
pub fn compile_clang(
    sy_path: &str,
    output_path: &str,
    opt_flag: bool,
    asm_flag: bool,
    ll_path: Option<String>,
) -> Result<(), CompilerError> {
    use errors::BackendError;

    let mut program = clang_frontend::Program::parse_file(sy_path)?;
    if opt_flag {
        clang_frontend::optimize(&mut program)?;
    }
    if let Some(ll_path) = ll_path {
        std::fs::write(ll_path, program.gen_ll().with_context(|| context!())?)
            .map_err(CompilerError::IOError)?;
    }
    let mut program = backend::from_llvm::gen_from_clang(&program)
        .map_err(|e| BackendError::GenFromLlvmError(format!("{e:?}")))?;
    if opt_flag {
        backend::optimize(&mut program)?;
    } else {
        backend::phisicalize(&mut program)?;
    }
    // check valid
    #[cfg(not(feature = "gen_virtual_asm"))]
    checker::ProgramChecker::check_valid(&checker::Riscv, &program);

    let asm = program.gen_asm();
    output(asm, output_path, asm_flag)
}

#[cfg(feature = "clang_enabled")]
pub fn compile_clang_llc(
    sy_path: &str,
    output_path: &str,
    opt_flag: bool,
    asm_flag: bool,
    ll_path: Option<String>,
) -> Result<(), CompilerError> {
    let mut program = clang_frontend::Program::parse_file(sy_path)?;
    if opt_flag {
        clang_frontend::optimize(&mut program)?;
    }
    if let Some(ll_path) = ll_path {
        std::fs::write(ll_path, program.gen_ll().with_context(|| context!())?)
            .map_err(CompilerError::IOError)?;
    }
    let mut program = clang_backend::gen_from_clang(&program)?;
    if opt_flag {
        clang_backend::optimize(&mut program);
    }
    let asm = program.gen_asm();
    output(asm, output_path, asm_flag)
}

#[cfg(feature = "clang_enabled")]
pub fn compile_self_llc(
    sy_path: &str,
    output_path: &str,
    opt_flag: bool,
    asm_flag: bool,
    ll_path: Option<String>,
) -> Result<(), CompilerError> {
    let content = std::fs::read_to_string(sy_path).map_err(CompilerError::IOError)?;
    let mut program = frontend::parse(&content)?;
    if opt_flag {
        frontend::optimize(&mut program);
    }
    let mut program = middle::gen(&program)?;
    if opt_flag {
        middle::optimize(&mut program);
    }
    // 中端接clang
    let llvm_ir = program.module.gen_llvm_ir();
    if let Some(ll_path) = ll_path {
        std::fs::write(ll_path, llvm_ir.clone()).with_context(|| context!())?;
    }
    let mut builder = tempfile::Builder::new();
    let tmp_llvm_file = builder.suffix(".ll").tempfile().unwrap();
    fs::write(&tmp_llvm_file, llvm_ir.as_bytes())?;
    let llvm = llvm_ir::Module::from_ir_path(&tmp_llvm_file).expect("llvm ir file not found");
    let program = clang_frontend::Program {
        tmp_llvm_file,
        llvm,
    };
    let mut program = clang_backend::gen_from_clang(&program)?;
    if opt_flag {
        clang_backend::optimize(&mut program);
    }
    let asm = program.gen_asm();
    output(asm, output_path, asm_flag)
}

fn output(asm: String, output_path: &str, asm_flag: bool) -> Result<(), CompilerError> {
    if !asm_flag {
        std::fs::write(output_path, asm2bin(asm)?).map_err(CompilerError::IOError)?;
        let mut permission = fs::metadata(output_path)?.permissions();
        permission.set_mode(0o755);
        fs::set_permissions(output_path, permission)?;
    } else {
        std::fs::write(output_path, asm).map_err(CompilerError::IOError)?;
    };
    Ok(())
}

#[allow(unused)]
pub fn asm2bin(asm: String) -> anyhow::Result<Vec<u8>> {
    // 使用lcc将汇编代码编译成二进制文件
    #[cfg(feature = "clang_enabled")]
    {
        let mut builder = tempfile::Builder::new();
        let tmp_asm_file = builder.suffix(".s").tempfile().unwrap();
        let tmp_bin_file = builder.suffix(".bin").tempfile().unwrap();
        let tmp_bin_path = tmp_bin_file.path();
        let tmp_asm_path = tmp_asm_file.path();
        std::fs::write(tmp_asm_path, asm).expect("msg: write asm failed");

        // let mut cmd = std::process::Command::new("llc");
        // cmd.arg("-o")
        //     .arg(tmp_bin_path)
        //     .arg(tmp_asm_path)
        //     .arg("-Wl,-Ttext=0x80000000");
        let mut cmd = std::process::Command::new("riscv64-linux-gnu-gcc-12");
        cmd.arg(tmp_asm_path).arg("-o").arg(tmp_bin_path);

        let output = cmd.output().expect("msg: exec llc failed");
        if !output.status.success() {
            panic!(
                "msg: exec llc failed\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let bin = std::fs::read(tmp_bin_path).expect("msg: read bin failed");
        Ok(bin)
    }
    #[cfg(not(feature = "clang_enabled"))]
    {
        Ok(asm.as_bytes().to_vec())
    }
}
