use super::*;
use super::{block::Block, gen_asm::GenTool};
use crate::config::CONFIG;
use rayon::prelude::*;

#[allow(unused)]
#[derive(Default)]
pub struct Func {
    name: String,
    args: Vec<String>,
    // stack_allocator,
    stack_allocator: Option<StackAllocator>,
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
            stack_allocator: None,
            other_bbs: Vec::new(),
            entry,
        }
    }
    pub fn stack_allocator(&self) -> Option<&StackAllocator> {
        self.stack_allocator.as_ref()
    }
    pub fn stack_allocator_mut(&mut self) -> &mut Option<StackAllocator> {
        &mut self.stack_allocator
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
        let other_bbs: Vec<&Block> = self.other_bbs.iter().collect();
        BBIter {
            entry: &self.entry,
            other_bbs,
            idx: 0,
        }
    }
}

pub struct BBIter<'a> {
    entry: &'a Block,
    other_bbs: Vec<&'a Block>,
    idx: usize,
}
impl<'a> Iterator for BBIter<'a> {
    type Item = &'a Block;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == 0 {
            self.idx += 1;
            Some(self.entry)
        } else {
            let ret = self.other_bbs.get(self.idx - 1).cloned();
            self.idx += 1;
            ret
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_iter_bb_order() {
        use super::*;
        let mut func = Func::default();
        let entry = Block::new("entry".to_string());
        let bb1 = Block::new("bb1".to_string());
        let bb2 = Block::new("bb4".to_string());
        let bb3 = Block::new("bb3".to_string());
        func.entry = entry;
        func.other_bbs.push(bb1);
        func.other_bbs.push(bb2);
        func.other_bbs.push(bb3);
        let mut iter = func.iter_bbs();
        assert_eq!(iter.next().unwrap().label(), "entry");
        assert_eq!(iter.next().unwrap().label(), "bb1");
        assert_eq!(iter.next().unwrap().label(), "bb4");
        assert_eq!(iter.next().unwrap().label(), "bb3");
        assert!(iter.next().is_none());
    }
}
