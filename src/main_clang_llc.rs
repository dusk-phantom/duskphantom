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

pub fn main() {
    #[cfg(feature = "clang_enabled")]
    {
        use clap::Parser;
        use duskphantom::cli::Cli;
        let cli = Cli::parse_from(vec!["c", "1.c", "-S", "-o", "1.s", "-O3"]);
        use duskphantom::compile_clang_llc;
        use duskphantom::errors::handle_error;
        use std::borrow::Borrow;
        let result = compile_clang_llc(&cli);
        if let Err(err) = result.borrow() {
            handle_error(err);
        }
    }
}
