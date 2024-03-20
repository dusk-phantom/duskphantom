use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub num_parallel_for_global_gen_asm: usize,
    pub num_parallel_for_func_gen_asm: usize,
    pub num_parallel_for_block_gen_asm: usize,
    pub num_parallel_for_inst_gen_asm: usize,
    pub reg_alloc_algo: String,
}

lazy_static! {
    pub static ref CONFIG: Config = {
        if let Ok(file) = std::fs::File::open("config.yaml") {
            let reader = std::io::BufReader::new(file);
            serde_yaml::from_reader(reader).unwrap()
        } else {
            Config {
                num_parallel_for_global_gen_asm: 12,
                num_parallel_for_block_gen_asm: 3,
                num_parallel_for_func_gen_asm: 2,
                num_parallel_for_inst_gen_asm: 2,
                reg_alloc_algo: "graph-coloring".to_string(),
            }
        }
    };
}
