use crate::config::CONFIG;

use super::{block::Block, gen_asm::GenTool};
use rayon::prelude::*;

#[allow(unused)]
pub struct Func {
    name: String,
    args: Vec<String>,
    // bacic blocks
    bbs: Vec<Block>,
    // sorted basic blocks by dict order of label,ascendingly
    sorted_bbs: Vec<Block>,
    // entry block
    entry: String,
}

impl Func {
    pub fn new() -> Func {
        Func {
            name: String::new(),
            args: Vec::new(),
            bbs: Vec::new(),
            sorted_bbs: Vec::new(),
            entry: String::new(),
        }
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn entry(&self) -> Option<&Block> {
        self.bbs.iter().find(|bb| bb.label() == self.entry)
    }
    pub fn gen_asm(&self) -> String {
        let entry = self.bbs.iter().find(|bb| bb.label() == self.entry).unwrap();
        let entry = entry.gen_asm();
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(CONFIG.num_parallel_for_block_gen_asm)
            .build()
            .unwrap();
        let mut other_bbs = self
            .bbs
            .iter()
            .filter(|bb| bb.label() != self.entry)
            .collect::<Vec<&Block>>();
        other_bbs.sort_by_cached_key(|bb| bb.label());
        let other_bbs = thread_pool.install(|| {
            other_bbs
                .par_iter()
                .map(|bb| bb.gen_asm())
                .collect::<Vec<String>>()
                .join("\n")
        });
        GenTool::gen_func(self.name.as_str(), entry.as_str(), other_bbs.as_str())
    }
}
