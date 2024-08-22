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

extern crate compiler;

fn main() {
    #[cfg(feature = "clang_enabled")]
    {
        use clap::Parser;
        use compiler::args::Cli;
        let cli = Cli::parse();
        start_compiler_cs(&cli);
    }
}
#[cfg(feature = "clang_enabled")]
fn start_compiler_cs(cli: &compiler::args::Cli) {
    use compiler::compile_clang;
    use compiler::errors::handle_error;
    use std::borrow::Borrow;
    let (sy_path, output_path, opt_flag, asm_flag, ll_path) = (
        &cli.sy,
        &cli.output,
        cli.optimize != 0,
        cli.asm,
        cli.ll.clone(),
    );
    let result = compile_clang(sy_path, output_path, opt_flag, asm_flag, ll_path);
    if let Err(err) = result.borrow() {
        handle_error(err);
    }
}
