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

use anyhow::{anyhow, Context, Result};
use duskphantom_utils::context;

use super::*;
use std::{fs, process::Command};

// 从clang frontend 生成riscv 汇编
pub struct Program {
    llvm_ir_s: String,
}

#[allow(unused)]
pub fn optimize(program: &mut Program, level: usize) -> Result<()> {
    let tmp_llvm_file = tempfile::Builder::new().suffix(".ll").tempfile().unwrap();
    fs::write(tmp_llvm_file.path(), program.llvm_ir_s.as_bytes()).unwrap();

    let tmp_opt_file = tempfile::Builder::new().suffix(".ll").tempfile().unwrap();

    let opt = match level {
        0 => "-O0",
        1 => "-O1",
        2 => "-O2",
        3 => "-O3",
        _ => "-O0",
    };
    let mut cmd = Command::new("opt");
    cmd.arg("-S")
        .arg(tmp_llvm_file.path())
        .arg("-o")
        .arg(tmp_opt_file.path())
        .arg(opt);
    let output = cmd
        .output()
        .map_err(|e| anyhow!("msg: exec opt failed: {}", e))
        .with_context(|| context!())?;
    if !output.status.success() {
        return Err(anyhow!(
            "msg: exec opt failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    program.llvm_ir_s = std::fs::read_to_string(tmp_opt_file.path())
        .map_err(|e| anyhow!("msg: read opt failed: {}", e))
        .with_context(|| context!())?;

    Ok(())
}

impl TryFrom<clang_frontend::Program> for Program {
    type Error = anyhow::Error;
    fn try_from(program: clang_frontend::Program) -> std::result::Result<Self, Self::Error> {
        Self::try_from(&program)
    }
}

impl TryFrom<&clang_frontend::Program> for Program {
    type Error = anyhow::Error;
    fn try_from(program: &clang_frontend::Program) -> std::result::Result<Self, Self::Error> {
        Ok(Program {
            llvm_ir_s: program.emit_llvm_ir().to_string(),
        })
    }
}

impl Program {
    pub fn parse_ll_code(ll_code: &str) -> Result<Self> {
        Ok(Program {
            llvm_ir_s: ll_code.to_string(),
        })
    }

    pub fn gen_asm(&mut self) -> Result<String> {
        let tmp_llvm_file = tempfile::Builder::new().suffix(".ll").tempfile()?;
        fs::write(tmp_llvm_file.path(), self.llvm_ir_s.as_bytes())
            .map_err(|e| anyhow!("msg: write llvm ir failed: {}", e))
            .with_context(|| context!())?;
        let tmp_asm_file = tempfile::Builder::new().suffix(".s").tempfile()?;
        let mut cmd = Command::new("llc");
        cmd.arg("-march=riscv64")
            .arg("-mattr=+m,+f,+d,+a,+c")
            .arg(tmp_llvm_file.path())
            .arg("-o")
            .arg(tmp_asm_file.path());

        let output = cmd
            .output()
            .map_err(|e| anyhow!("msg: exec llc failed: {}", e))
            .with_context(|| context!())?;
        if !output.status.success() {
            return Err(anyhow!(
                "msg: exec llc failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        std::fs::read_to_string(tmp_asm_file.path())
            .map_err(|e| anyhow!("msg: read asm failed: {}", e))
            .with_context(|| context!())
    }
}
