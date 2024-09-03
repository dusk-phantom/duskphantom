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

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub num_parallel_for_global_gen_asm: usize,
    pub num_parallel_for_func_gen_asm: usize,
    pub num_parallel_for_block_gen_asm: usize,
    pub reg_alloc_algo: String,
    pub open_auto_parallel: bool,
}

lazy_static! {
    pub static ref CONFIG: Config = {
        if let Ok(file) = std::fs::File::open("config.yaml") {
            let reader = std::io::BufReader::new(file);
            serde_yaml::from_reader(reader).unwrap()
        } else {
            use std::env;
            Config {
                num_parallel_for_global_gen_asm: env::var("NUM_PARALLEL_FOR_GLOBAL_GEN_ASM")
                    .unwrap_or_else(|_| "12".to_string())
                    .parse()
                    .unwrap_or(12),
                num_parallel_for_func_gen_asm: env::var("NUM_PARALLEL_FOR_FUNC_GEN_ASM")
                    .unwrap_or_else(|_| "4".to_string())
                    .parse()
                    .unwrap_or(4),
                num_parallel_for_block_gen_asm: env::var("NUM_PARALLEL_FOR_BLOCK_GEN_ASM")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()
                    .unwrap_or(3),
                reg_alloc_algo: env::var("REG_ALLOC_ALGO")
                    .unwrap_or_else(|_| "graph-coloring".to_string()),
                open_auto_parallel: env::var("OPEN_AUTO_PARALLEL")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(false),
            }
        }
    };
}
