// Copyright 2024 Duskphantom Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;

use duskphantom_utils::context;

use clang_front_back::clang_backend;
use clang_front_back::clang_frontend;
use errors::CompilerError;
use std::fs;
use std::os::unix::fs::PermissionsExt;
pub mod cli;
use cli::Cli;

pub mod config;
pub mod errors;
pub use duskphantom_backend as backend;
pub use duskphantom_frontend as frontend;
pub use duskphantom_middle as middle;

use clap::arg;

/// compile sysy source code to rv64gc asm
pub fn compile(cli: &Cli) -> Result<(), CompilerError> {
    let content = std::fs::read_to_string(&cli.sy).map_err(CompilerError::IOError)?;
    let mut program = frontend::parse(&content)?;
    if cli.optimize != 0 {
        frontend::optimize(&mut program, cli.optimize);
    }

    let mut program = middle::Program::try_from(program)?;
    if cli.optimize != 0 {
        middle::optimize(&mut program, cli.optimize);
    }
    if let Some(ll_path) = cli.ll.as_ref() {
        std::fs::write(ll_path, program.module.gen_llvm_ir()).with_context(|| context!())?;
    }
    let mut program = backend::from_self::gen_from_self(&program)?;

    if cli.optimize != 0 {
        backend::optimize(&mut program)?;
    } else {
        backend::phisicalize(&mut program)?;
    }

    let asm = program.gen_asm();
    output(asm, &cli.output, cli.asm)
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
    use duskphantom_backend::BackendError;
    let mut program = clang_frontend::Program::parse_c_file(sy_path)?;
    if opt_flag {
        clang_frontend::optimize(&mut program, 3)?;
    }
    if let Some(ll_path) = ll_path {
        std::fs::write(ll_path, program.emit_llvm_ir()).map_err(CompilerError::IOError)?;
    }
    let mut program = backend::from_llvm::gen_from_clang(&program)
        .map_err(|e| BackendError::GenFromLlvmError(format!("{e:?}")))?;
    if opt_flag {
        backend::optimize(&mut program)?;
    } else {
        backend::phisicalize(&mut program)?;
    }

    let asm = program.gen_asm();
    output(asm, output_path, asm_flag)
}

#[cfg(feature = "clang_enabled")]
pub fn compile_clang_llc(cli: &Cli) -> Result<(), CompilerError> {
    let mut program = clang_frontend::Program::parse_c_file(&cli.sy)?;

    if cli.optimize != 0 {
        clang_frontend::optimize(&mut program, cli.optimize)?;
    }

    if let Some(ll_path) = &cli.ll {
        std::fs::write(ll_path, program.emit_llvm_ir()).map_err(CompilerError::IOError)?;
    }
    let mut program = clang_backend::Program::try_from(&program)?;

    if cli.optimize != 0 {
        clang_backend::optimize(&mut program, cli.optimize)?;
    }

    let asm = program.gen_asm()?;
    output(asm, &cli.output, cli.asm)
}

#[cfg(feature = "clang_enabled")]
pub fn compile_self_llc(cli: &Cli) -> Result<(), CompilerError> {
    let content = std::fs::read_to_string(cli.sy.as_str()).map_err(CompilerError::IOError)?;
    let mut program = frontend::parse(&content)?;
    if cli.optimize != 0 {
        frontend::optimize(&mut program, cli.optimize);
    }

    let mut program = middle::Program::try_from(program)?;
    if cli.optimize != 0 {
        middle::optimize(&mut program, cli.optimize);
    }

    // 中端接clang
    let llvm_ir = program.module.gen_llvm_ir();
    if let Some(ll_path) = cli.ll.as_ref() {
        std::fs::write(ll_path, llvm_ir.clone()).with_context(|| context!())?;
    }
    let program = clang_frontend::Program::parse_ll_code(&llvm_ir)?;
    let mut program = clang_backend::Program::try_from(&program)?;

    if cli.optimize != 0 {
        clang_backend::optimize(&mut program, cli.optimize)?;
    }
    let asm = program.gen_asm()?;
    output(asm, &cli.output, cli.asm)
}

fn output(asm: String, output_path: &str, asm_flag: bool) -> Result<(), CompilerError> {
    if !asm_flag {
        std::fs::write(output_path, gcc_asm2bin(asm)?).map_err(CompilerError::IOError)?;
        let mut permission = fs::metadata(output_path)?.permissions();
        permission.set_mode(0o755);
        fs::set_permissions(output_path, permission)?;
    } else {
        std::fs::write(output_path, asm).map_err(CompilerError::IOError)?;
    };
    Ok(())
}

#[allow(unused)]
pub fn gcc_asm2bin(asm: String) -> anyhow::Result<Vec<u8>> {
    // 使用riskv64-linux-gnu-gcc编译
    let mut builder = tempfile::Builder::new();
    let tmp_asm_file = builder.suffix(".s").tempfile().unwrap();
    let tmp_bin_file = builder.suffix(".bin").tempfile().unwrap();
    let tmp_bin_path = tmp_bin_file.path();
    let tmp_asm_path = tmp_asm_file.path();
    std::fs::write(tmp_asm_path, asm).expect("msg: write asm failed");

    let mut cmd = std::process::Command::new("riscv64-linux-gnu-gcc-12");
    cmd.arg(tmp_asm_path).arg("-o").arg(tmp_bin_path);

    let output = cmd
        .output()
        .expect("msg: exec riskv64-linux-gnu-gcc failed");
    if !output.status.success() {
        panic!(
            "msg: exec riskv64-linux-gnu-gcc failed\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let bin = std::fs::read(tmp_bin_path).expect("msg: read bin failed");
    Ok(bin)
}
