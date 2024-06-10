use errors::CompilerError;

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

use clap::{arg, App};

/// compile sysy source code to rv64gc asm
pub fn compile(
    sy_path: &str,
    output_path: &str,
    opt_flag: bool,
    asm_flag: bool,
) -> Result<(), CompilerError> {
    let mut program = frontend::parse(sy_path)?;
    if opt_flag {
        frontend::optimize(&mut program);
    }
    let mut program = middle::gen(&program)?;
    if opt_flag {
        middle::optimize(&mut program);
    }
    let mut program = backend::gen(&program)?;
    if opt_flag {
        backend::optimize(&mut program);
    }
    let asm = program.gen_asm();
    let output = if !asm_flag { asm2bin(asm) } else { asm };
    std::fs::write(output_path, output).map_err(CompilerError::IOError)
}

#[cfg(feature = "clang_enabled")]
/// compile from clang
pub fn compile_clang(
    src_file: &str,
    output_file: &str,
    opt_flag: bool,
    asm_flag: bool,
    ll_path: Option<String>,
) -> Result<(), CompilerError> {
    let mut program = clang_frontend::Program::parse(src_file);
    if opt_flag {
        clang_frontend::optimize(&mut program);
    }
    if let Some(ll_path) = ll_path {
        std::fs::write(ll_path, program.gen_ll()).map_err(CompilerError::IOError)?;
    }
    let mut program = backend::gen_from_clang(&program)?;
    if opt_flag {
        backend::optimize(&mut program);
    } else {
        backend::phisicalize(&mut program);
    }
    let asm = program.gen_asm();
    let output = if !asm_flag { asm2bin(asm) } else { asm };
    std::fs::write(output_file, output).map_err(CompilerError::IOError)
}

#[cfg(feature = "clang_enabled")]
pub fn compile_clang_llc(
    src_file: &str,
    output_file: &str,
    opt_flag: bool,
    asm_flag: bool,
    ll_path: Option<String>,
) -> Result<(), CompilerError> {
    let mut program = clang_frontend::Program::parse(src_file);
    if opt_flag {
        clang_frontend::optimize(&mut program);
    }
    if let Some(ll_path) = ll_path {
        std::fs::write(ll_path, program.gen_ll()).map_err(CompilerError::IOError)?;
    }
    let mut program = clang_backend::gen_from_clang(&program)?;
    if opt_flag {
        clang_backend::optimize(&mut program);
    }
    let asm = program.gen_asm();
    let output = if !asm_flag { asm2bin(asm) } else { asm };
    std::fs::write(output_file, output).map_err(CompilerError::IOError)
}

#[allow(unused)]
pub fn asm2bin(asm: String) -> String {
    // 使用lcc将汇编代码编译成二进制文件
    #[cfg(feature = "clang_enabled")]
    {
        let mut builder = tempfile::Builder::new();
        let tmp_asm_file = builder.suffix(".s").tempfile().unwrap();
        let tmp_bin_file = builder.suffix(".bin").tempfile().unwrap();
        let tmp_bin_path = tmp_bin_file.path();
        let tmp_asm_path = tmp_asm_file.path();
        std::fs::write(tmp_asm_path, asm).expect("msg: write asm failed");
        let mut cmd = std::process::Command::new("lcc");
        cmd.arg("-o")
            .arg(tmp_bin_path)
            .arg(tmp_asm_path)
            .arg("-Wl,-Ttext=0x80000000");
        let output = cmd.output().expect("msg: exec lcc failed");
        if !output.status.success() {
            panic!("msg: exec lcc failed");
        }
        let bin = std::fs::read(tmp_bin_path).expect("msg: read bin failed");
        let mut bin_str = String::new();
        for byte in bin {
            bin_str.push_str(&format!("{:02x}", byte));
        }
        bin_str
    }
    #[cfg(not(feature = "clang_enabled"))]
    {
        String::new()
    }
}
