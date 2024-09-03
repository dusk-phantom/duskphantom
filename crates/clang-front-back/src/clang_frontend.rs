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

// use crate::context;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use duskphantom_utils::context;
use llvm_ir::Module;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct Program {
    llvm_ir_s: String,
    pub llvm: Module,
}

impl Program {
    pub fn parse_c_file<P: AsRef<Path>>(file: P) -> Result<Self> {
        let file = file
            .as_ref()
            .to_str()
            .ok_or(anyhow!("msg: file path error"))
            .with_context(|| context!())?;
        let mut builder = tempfile::Builder::new();
        let tmp_llvm_file = builder.suffix(".ll").tempfile().unwrap();
        let mut cmd = Command::new("clang");
        // clang -S -emit-llvm -Xclang -disable-O0-optnone -target riscv64 -x c 1.c -o 1.ll
        cmd.arg("-S")
            .arg("-emit-llvm")
            .arg("-Wno-implicit-function-declaration")
            .arg("-Xclang")
            .arg("-disable-O0-optnone")
            .arg("-target")
            .arg("riscv64")
            .arg("-x")
            .arg("c")
            // 允许使用非.c后缀的文件名
            .arg(file)
            .arg("-o")
            .arg(tmp_llvm_file.path());
        let output = cmd.output().expect("msg: exec clang failed");

        if !output.status.success() {
            bail!(
                "msg: exec clang failed,Error:{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        // 使用llvm_ir crate从.ll内容文件中读取llvm ir
        let llvm_ir_string =
            fs::read_to_string(tmp_llvm_file.path()).expect("msg: read llvm ir file failed");
        let llvm = Module::from_ir_path(tmp_llvm_file.path()).expect("msg: parse llvm ir failed");
        Ok(Self {
            llvm_ir_s: llvm_ir_string,
            llvm,
        })
    }

    pub fn parse_ll_code(ll_code: &str) -> Result<Self> {
        let mut builder = tempfile::Builder::new();
        let tmp_llvm_file = builder.suffix(".ll").tempfile().unwrap();
        fs::write(tmp_llvm_file.path(), ll_code).expect("msg: write llvm ir file failed");
        let llvm = Module::from_ir_path(tmp_llvm_file.path()).expect("msg: parse llvm ir failed");
        Ok(Self {
            llvm_ir_s: ll_code.to_string(),
            llvm,
        })
    }

    #[deprecated]
    /// use emit_llvm_ir instead
    pub fn gen_ll(&self) -> anyhow::Result<String> {
        Ok(self.llvm_ir_s.to_string())
    }

    pub fn emit_llvm_ir(&self) -> &str {
        &self.llvm_ir_s
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

pub fn optimize(program: &mut Program, level: usize) -> anyhow::Result<()> {
    // 使用clang 命令优化.ll 代码
    let level = match level {
        0 => "-O0",
        1 => "-O1",
        2 => "-O2",
        3 => "-O3",
        _ => "-O0",
    };
    let tmp_llvm_file = tempfile::Builder::new().suffix(".ll").tempfile()?;
    fs::write(tmp_llvm_file.path(), program.emit_llvm_ir())?;

    let tmp_opt_file = tempfile::Builder::new().suffix(".ll").tempfile()?;
    let mut cmd = Command::new("opt");
    cmd.arg("-S")
        .arg(tmp_llvm_file.path())
        .arg("-o")
        .arg(tmp_opt_file.path())
        .arg(level);

    let output = cmd
        .output()
        .map_err(|e| anyhow!("{e}"))
        .with_context(|| context!())?;

    if !output.status.success() {
        return Err(anyhow!(
            "msg: exec opt failed,Error:{}",
            String::from_utf8_lossy(&output.stderr)
        ))
        .with_context(|| context!());
    }

    let llvm = Module::from_ir_path(tmp_opt_file.path())
        .map_err(|e| anyhow!("{e}"))
        .with_context(|| context!())?;
    program.llvm = llvm;
    Ok(())
}
