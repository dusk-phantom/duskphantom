use crate::config::CONFIG;

use super::{block::Block, gen_asm::GenTool};
use rayon::prelude::*;

#[allow(unused)]
#[derive(Default)]
pub struct Func {
    name: String,
    args: Vec<String>,
    // entry block
    entry: Block,
    // basic blocks
    other_bbs: Vec<Block>,
}

impl Func {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn entry(&self) -> &Block {
        &self.entry
    }
    pub fn gen_asm(&self) -> String {
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(CONFIG.num_parallel_for_block_gen_asm)
            .build()
            .unwrap();
        let bbs: Vec<&Block> = self.iter_bbs().collect();
        let bbs_asm = thread_pool.install(|| {
            bbs.par_iter()
                .map(|bb| bb.gen_asm())
                .collect::<Vec<String>>()
                .join("\n")
        });
        GenTool::gen_func(self.name.as_str(), &bbs_asm)
    }
}

impl Func {
    pub fn new(name: String, args: Vec<String>, entry: Block) -> Func {
        Func {
            name,
            args,
            other_bbs: Vec::new(),
            entry,
        }
    }
    pub fn push_bb(&mut self, bb: Block) {
        self.other_bbs.push(bb);
    }
    pub fn extend_bbs(&mut self, bbs: Vec<Block>) {
        self.other_bbs.extend(bbs);
    }

    // iter bbs in a special order,in which entry is the first one,
    // and other bbs in order of label's dictionary order
    pub fn iter_bbs(&self) -> BBIter {
        let mut ordered_bbs = Vec::new();
        let mut ordered_other_bbs: Vec<&Block> = self.other_bbs.iter().collect();
        ordered_other_bbs.sort_by_key(|b| b.label());
        ordered_bbs.extend(ordered_other_bbs.iter());
        BBIter {
            entry: &self.entry,
            ordered_other_bbs: ordered_bbs,
            idx: 0,
        }
    }

    // count stack_size this func need
    // pub fn stack_size(&self) -> usize {
    //     let mut size = 0;
    //     self.iter_bbs().for_each(|bb| {
    //         bb.insts().iter().for_each(|inst| {
    //             use Inst::*;
    //             let stack_slot=
    //             match inst{
    //                 Ld(inst)=> inst.get_stack_slot(),
    //                 Lw(inst)=> inst.get_stack_slot(),
    //                 Sw(inst)=> inst.get_stack_slot(),
    //                 Sd(inst)=> inst.get_stack_slot(),
    //             };
    //             todo!();
    //         })
    //     });
    //     0
    // }
}

pub struct BBIter<'a> {
    entry: &'a Block,
    ordered_other_bbs: Vec<&'a Block>,
    idx: usize,
}
impl<'a> Iterator for BBIter<'a> {
    type Item = &'a Block;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == 0 {
            self.idx += 1;
            Some(self.entry)
        } else {
            let ret = self.ordered_other_bbs.get(self.idx - 1).cloned();
            self.idx += 1;
            ret
        }
    }
}
