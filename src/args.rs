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

use clap::Parser;

use super::*;

#[derive(Parser, Debug)]
#[command(version,about,long_about=None)]
pub struct Cli {
    pub sy: String,
    #[arg(short = 'O', long, default_value = "0")]
    pub optimize: i32,
    #[arg(short = 'S', long)]
    pub asm: bool,
    #[arg(short = 'o', long, value_name = "output")]
    pub output: String,
    #[arg(short, long, value_name = "llvm_path")]
    pub ll: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    static BIN: &str = "compiler";
    #[test]
    fn test_normal() {
        let cli = super::Cli::parse_from([BIN, "1.sy", "-S", "-o", "1.s"]);
        dbg!(&cli);
        assert_eq!(cli.sy, "1.sy");
        assert_eq!(cli.output, "1.s");
        assert_eq!(cli.optimize, 0);
        assert!(cli.asm);
        assert_eq!(cli.ll, None);
    }
    #[test]
    fn test_optimize() {
        let cli = super::Cli::parse_from([BIN, "1.sy", "-S", "-o", "1.s", "-O1"]);
        // dbg!(&cli);
        assert_eq!(cli.sy, "1.sy");
        assert_eq!(cli.output, "1.s");
        assert_eq!(cli.optimize, 1);
        assert!(cli.asm);
        assert_eq!(cli.ll, None);
    }

    #[test]
    fn test_ll() {
        let cli = super::Cli::parse_from([BIN, "1.sy", "-S", "-o", "1.s", "--ll", "1.ll"]);
        dbg!(&cli);
        assert_eq!(cli.sy, "1.sy");
        assert_eq!(cli.output, "1.s");
        assert_eq!(cli.optimize, 0);
        assert!(cli.asm);
        assert_eq!(cli.ll, Some("1.ll".to_string()));
    }
}
