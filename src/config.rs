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
