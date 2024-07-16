use std::collections::HashMap;
use std::usize;

use super::*;
use super::{block::Block, gen_asm::GenTool};
use crate::config::CONFIG;
use rayon::prelude::*;

#[allow(unused)]
#[derive(Default, Debug)]
pub struct Func {
    name: String,
    args: Vec<String>,
    ret: Option<Reg>,
    /// the size of stack where extra args are stored
    caller_regs_stack: Option<u32>,
    /// the max size of stack where callee's args are stored
    max_callee_regs_stack: Option<u32>,
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
    /// create a new function, default return type is void
    pub fn new(name: String, args: Vec<String>, entry: Block) -> Func {
        Func {
            name,
            args,
            ret: None,
            caller_regs_stack: None,
            max_callee_regs_stack: None,
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

    pub fn caller_regs_stack(&self) -> u32 {
        self.caller_regs_stack.unwrap_or(0)
    }
    pub fn caller_regs_stack_mut(&mut self) -> &mut Option<u32> {
        &mut self.caller_regs_stack
    }
    pub fn max_callee_regs_stack(&self) -> u32 {
        self.max_callee_regs_stack.unwrap_or(0)
    }
    pub fn max_callee_regs_stack_mut(&mut self) -> &mut Option<u32> {
        &mut self.max_callee_regs_stack
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
    /// check if there is a call instruction in the function
    pub fn is_caller(f: &Func) -> bool {
        for bb in f.iter_bbs() {
            for inst in bb.insts() {
                if let Inst::Call { .. } = inst {
                    return true;
                }
            }
        }
        false
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
/// impl Some functionality for reg alloc
impl Func {
    /// compute the in and out set of each basic block
    /// return a tuple of two hashmaps, the first one is in set, the second one is out set
    pub fn in_out_bbs(f: &Func) -> Result<(InBBs, OutBBs)> {
        let mut outs: HashMap<String, Vec<String>> = HashMap::new();
        for bb in f.iter_bbs() {
            let mut rev_iter = bb.insts().iter().rev();
            let mut to_bbs: Vec<String> = vec![];
            if let Some(last_inst) = rev_iter.next() {
                match last_inst {
                    Inst::Ret { .. } => {}
                    Inst::Jmp(jmp) => {
                        to_bbs.push(jmp.to_bb()?.to_string());
                        if let Some(last2) = rev_iter.next() {
                            match last2 {
                                Inst::Beq(beq) => {
                                    to_bbs.push(beq.label().to_string());
                                }
                                Inst::Bge(bge) => {
                                    to_bbs.push(bge.label().to_string());
                                }
                                Inst::Bgt(bgt) => {
                                    to_bbs.push(bgt.label().to_string());
                                }
                                Inst::Ble(ble) => {
                                    to_bbs.push(ble.label().to_string());
                                }
                                Inst::Blt(blt) => {
                                    to_bbs.push(blt.label().to_string());
                                }
                                Inst::Bne(bne) => {
                                    to_bbs.push(bne.label().to_string());
                                }
                                Inst::Jmp(_) => {
                                    unreachable!("The last two instructions of a basic block should not be both jump instructions");
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {
                        unreachable!("The last instruction of a basic block should be a return or jump instruction");
                    }
                }
            }
            outs.insert(bb.label().to_string(), to_bbs);
        }
        let mut ins: HashMap<String, Vec<String>> = HashMap::new();
        for bb in f.iter_bbs() {
            let to_bbs = outs.get(bb.label()).unwrap();
            for to_bb in to_bbs {
                if let Some(ins_to_bb) = ins.get_mut(to_bb) {
                    ins_to_bb.push(bb.label().to_string());
                } else {
                    ins.insert(to_bb.to_string(), vec![bb.label().to_string()]);
                }
            }
            if !ins.contains_key(bb.label()) {
                ins.insert(bb.label().to_string(), vec![]);
            }
        }
        let bbs: HashMap<String, &Block> = f
            .iter_bbs()
            .map(|bb| (bb.label().to_string(), bb))
            .collect();
        let ins = InBBs {
            bbs: bbs.clone(),
            ins,
        };
        let outs = OutBBs { bbs, outs };

        Ok((ins, outs))
    }
}

#[derive(Debug)]
pub struct InBBs<'a> {
    bbs: HashMap<String, &'a Block>,
    ins: HashMap<String, Vec<String>>,
}
#[derive(Debug)]
pub struct OutBBs<'a> {
    bbs: HashMap<String, &'a Block>,
    outs: HashMap<String, Vec<String>>,
}
impl<'a> InBBs<'a> {
    pub fn ins(&'a self, bb: &Block) -> Vec<&'a Block> {
        self.ins
            .get(bb.label())
            .unwrap()
            .iter()
            .map(|label| self.bbs[label])
            .collect()
    }
}
impl<'a> OutBBs<'a> {
    pub fn outs(&'a self, bb: &Block) -> Vec<&'a Block> {
        self.outs
            .get(bb.label())
            .unwrap()
            .iter()
            .map(|label| self.bbs[label])
            .collect()
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
    use crate::backend::{Block, Func};

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
