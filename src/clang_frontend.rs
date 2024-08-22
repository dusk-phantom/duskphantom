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

use crate::context;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use llvm_ir::Module;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

pub struct Program {
    pub tmp_llvm_file: NamedTempFile,
    pub llvm: Module,
}

impl Program {
    pub fn parse_file<P: AsRef<Path>>(file: P) -> Result<Self> {
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
            // 制定使用非.c后缀的文件名
            .arg(file)
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
        Ok(Self {
            tmp_llvm_file,
            llvm,
        })
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
