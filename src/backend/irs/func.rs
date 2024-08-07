use std::collections::HashSet;

use super::*;
use super::{block::Block, gen_asm::GenTool};

use crate::config::CONFIG;
use rayon::prelude::*;

#[allow(unused)]
#[derive(Default, Debug)]
#[non_exhaustive]
pub struct Func {
    name: String,
    args: Vec<String>,
    ret: Option<Reg>,
    reg_gener: Option<RegGenerator>,
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
    pub fn entry_mut(&mut self) -> &mut Block {
        &mut self.entry
    }

    pub fn gen_asm(&self) -> String {
        if CONFIG.num_parallel_for_func_gen_asm == 1 {
            println!("num_parallel_for_func_gen_asm == 1");
            let mut bbs_asm = String::with_capacity(256);
            for bb in self.iter_bbs() {
                bbs_asm.push_str(bb.gen_asm().as_str());
                bbs_asm.push('\n');
            }
            return GenTool::gen_func(self.name.as_str(), bbs_asm.as_str());
        }
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
    /// create a new function, default return type is void
    pub fn new(name: String, args: Vec<String>, entry: Block) -> Func {
        Func {
            name,
            args,
            ret: None,
            reg_gener: None,
            stack_allocator: None,
            other_bbs: Vec::new(),
            entry,
        }
    }

    pub fn ret(&self) -> Option<&Reg> {
        self.ret.as_ref()
    }
    pub fn ret_mut(&mut self) -> &mut Option<Reg> {
        &mut self.ret
    }

    pub fn stack_allocator(&self) -> Option<&StackAllocator> {
        self.stack_allocator.as_ref()
    }

    pub fn stack_allocator_mut(&mut self) -> &mut Option<StackAllocator> {
        &mut self.stack_allocator
    }

    pub fn reg_gener(&self) -> Option<&RegGenerator> {
        self.reg_gener.as_ref()
    }

    pub fn reg_gener_mut(&mut self) -> &mut Option<RegGenerator> {
        &mut self.reg_gener
    }

    pub fn push_bb(&mut self, bb: Block) {
        self.other_bbs.push(bb);
    }

    pub fn extend_bbs(&mut self, bbs: Vec<Block>) {
        self.other_bbs.extend(bbs);
    }

    /// check if there is a call instruction in the function
    pub fn is_caller(&self) -> bool {
        for bb in self.iter_bbs() {
            for inst in bb.insts() {
                if let Inst::Call { .. } = inst {
                    return true;
                }
            }
        }
        false
    }

    /// get all virtual regs in the function
    pub fn v_regs(&self) -> HashSet<Reg> {
        self.regs().into_iter().filter(|r| r.is_virtual()).collect()
    }

    pub fn regs(&self) -> HashSet<Reg> {
        let mut regs = HashSet::new();
        for bb in self.iter_bbs() {
            for inst in bb.insts() {
                regs.extend(inst.uses().iter().cloned());
                regs.extend(inst.defs().iter().cloned());
            }
        }
        regs
    }

    // iter bbs in a special order,in which entry is the first one
    pub fn iter_bbs(&self) -> BBIter {
        let other_bbs: Vec<&Block> = self.other_bbs.iter().collect();
        BBIter {
            entry: &self.entry,
            other_bbs,
            idx: 0,
        }
    }

    /// iter bbs in a special order mutably, in which entry is the first one,
    pub fn iter_bbs_mut(&mut self) -> impl Iterator<Item = &mut Block> {
        let mut bbs = vec![&mut self.entry];
        bbs.extend(self.other_bbs.iter_mut());
        bbs.into_iter()
    }

    // get exit bbs, which are the bbs that end with ret or tail call
    pub fn exit_bbs(&self) -> Vec<&Block> {
        let mut ret = vec![];
        for bb in self.iter_bbs() {
            let insts = bb.insts();
            if let Some(last_inst) = insts.last() {
                match last_inst {
                    Inst::Ret { .. } => {
                        ret.push(bb);
                    }
                    Inst::Tail { .. } => {
                        ret.push(bb);
                    }
                    _ => {}
                }
            }
        }
        ret
    }

    // get all the bbs that end with ret or tail call in mutable way
    pub fn exit_bbs_mut(&mut self) -> impl Iterator<Item = &mut Block> {
        let mut ret = vec![];
        for bb in self.iter_bbs_mut() {
            let insts = bb.insts();
            if let Some(last_inst) = insts.last() {
                match last_inst {
                    Inst::Ret { .. } => {
                        ret.push(bb);
                    }
                    Inst::Tail { .. } => {
                        ret.push(bb);
                    }
                    _ => {}
                }
            }
        }
        ret.into_iter()
    }

    pub fn merge_bb(&mut self, from: &str, to: &str) -> Result<()> {
        fn remove_to(vec: &mut Vec<Block>, to: &str) -> Result<Block> {
            let idx = vec.iter().position(|bb| bb.label() == to);
            if let Some(idx) = idx {
                Ok(vec.remove(idx))
            } else {
                Err(anyhow!(""))
            }
        }
        let to = remove_to(&mut self.other_bbs, to)?;
        let from = if self.entry().label() == from {
            self.entry_mut()
        } else {
            self.iter_bbs_mut()
                .find(|bb| bb.label() == from)
                .ok_or(anyhow!(""))
                .with_context(|| context!())?
        };
        from.merge(to)?;
        Ok(())
    }
}

pub struct BBDistanceCounter {
    num_insts: Vec<(String, usize)>,
}
impl BBDistanceCounter {
    /// FIXME: test needed
    pub fn distance_between(&self, from: &str, to: &str) -> Option<usize> {
        let from_idx = self.num_insts.iter().position(|(label, _)| label == from)?;
        let to_idx = self.num_insts.iter().position(|(label, _)| label == to)?;
        if from_idx < to_idx {
            let distance = self.num_insts[(from_idx + 1)..to_idx]
                .iter()
                .map(|(_, num_inst)| *num_inst)
                .sum();
            Some(distance)
        } else {
            let distance = self.num_insts[to_idx..=from_idx]
                .iter()
                .map(|(_, num_inst)| *num_inst)
                .sum();
            Some(distance)
        }
    }
}
/// helper method for handling long jmp
impl Func {
    pub fn bb_distances(&self) -> BBDistanceCounter {
        let num_insts: Vec<(String, usize)> = self
            .iter_bbs()
            .map(|bb| (bb.label().to_string(), bb.insts().len()))
            .collect();

        BBDistanceCounter { num_insts }
    }
}

#[derive(Clone)]
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
    use super::*;

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

    #[test]
    fn test_func_new() {
        let func = Func::new("main".to_string(), vec![], Block::new("entry".to_string()));
        assert_eq!(func.name(), "main");
        assert_eq!(func.args.len(), 0);
        assert_eq!(func.entry().label(), "entry");
        assert_eq!(func.ret(), None); // default return type is void
    }
}
