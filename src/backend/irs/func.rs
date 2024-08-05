use std::collections::{HashMap, HashSet};

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

    /// compute the live in and live out set of regs of each basic block
    pub fn reg_lives<'a>(f: &'a Func, ins: &InBBs<'a>, outs: &OutBBs<'a>) -> Result<RegLives> {
        let mut live_ins: HashMap<String, HashSet<Reg>> = HashMap::new();
        let mut live_outs: HashMap<String, HashSet<Reg>> = HashMap::new();

        // consider the exit block
        if let Some(ret) = f.ret() {
            for bb in f.exit_bbs() {
                let mut live_out: HashSet<Reg> = HashSet::new();
                live_out.insert(*ret);
                live_outs.insert(bb.label().to_string(), live_out);
            }
        }
        let bb_iter = f.iter_bbs();

        // consider live_use
        for bb in bb_iter.clone() {
            let mut live_in: HashSet<Reg> = HashSet::new();
            let mut defed_regs: HashSet<Reg> = HashSet::new();
            for inst in bb.insts().iter().rev() {
                for reg in inst.uses() {
                    if !defed_regs.contains(reg) {
                        live_in.insert(*reg);
                    }
                }
                defed_regs.extend(inst.defs());
            }
            live_ins.insert(bb.label().to_string(), live_in);
        }

        // loop to compute live_in and live_out
        // FIXME: 使用位图实现的寄存器记录表来加速运算过程，以及节省内存
        let mut changed = true;
        while changed {
            changed = false;
            for bb in bb_iter.clone() {
                let mut new_live_in = live_ins.get(bb.label()).cloned().unwrap_or(HashSet::new());
                for in_bb in ins.ins(bb) {
                    if let Some(out) = live_outs.get(in_bb.label()) {
                        new_live_in.extend(out.iter().cloned());
                    }
                }
                let mut new_live_out = live_outs.get(bb.label()).cloned().unwrap_or(HashSet::new());
                for out_bb in outs.outs(bb) {
                    if let Some(in_) = live_ins.get(out_bb.label()) {
                        new_live_out.extend(in_.iter().cloned());
                    }
                }
                if !live_ins.contains_key(bb.label()) || new_live_in != live_ins[bb.label()] {
                    live_ins.insert(bb.label().to_string(), new_live_in);
                    changed = true;
                }
                if !live_outs.contains_key(bb.label()) || new_live_out != live_outs[bb.label()] {
                    live_outs.insert(bb.label().to_string(), new_live_out);
                    changed = true;
                }
            }
        }

        Ok(RegLives {
            live_ins,
            live_outs,
        })
    }

    /// compute the reg interference graph of a function
    pub fn reg_interfere_graph(f: &Func) -> Result<HashMap<Reg, HashSet<Reg>>> {
        let mut graph: HashMap<Reg, HashSet<Reg>> = HashMap::new();

        // for each physical register, add it to the graph
        let p_regs = Reg::physical_regs();
        for p_reg in p_regs {
            graph.insert(*p_reg, HashSet::new());
            for other_p_reg in p_regs {
                if p_reg != other_p_reg {
                    graph.entry(*p_reg).or_default().insert(*other_p_reg);
                }
            }
        }

        // for each basic block, collect interference between regs
        let (ins, outs) = Func::in_out_bbs(f)?;
        let reg_lives = Func::reg_lives(f, &ins, &outs)?;
        // FIXME: 使用位图实现的寄存器记录表来加速运算过程，以及节省内存
        for bb in f.iter_bbs() {
            let mut alive_regs: HashSet<Reg> = reg_lives.live_outs(bb).clone();
            for r in alive_regs.iter() {
                if !graph.contains_key(r) {
                    graph.insert(*r, HashSet::new());
                }
            }
            for inst in bb.insts().iter().rev() {
                let defs = inst.defs();
                for reg in defs.clone() {
                    alive_regs.remove(reg);
                    for alive_reg in alive_regs.iter() {
                        graph.entry(*reg).or_default().insert(*alive_reg);
                        graph.entry(*alive_reg).or_default().insert(*reg);
                    }
                    alive_regs.insert(*reg);
                }
                alive_regs.retain(|r| !defs.contains(&r));
                for reg in inst.uses() {
                    alive_regs.insert(*reg);
                    for alive_reg in alive_regs.iter() {
                        graph.entry(*reg).or_default().insert(*alive_reg);
                        graph.entry(*alive_reg).or_default().insert(*reg);
                    }
                }
            }
        }

        Ok(graph)
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

pub struct RegLives {
    live_ins: HashMap<String, HashSet<Reg>>,
    live_outs: HashMap<String, HashSet<Reg>>,
}
impl RegLives {
    pub fn live_ins(&self, bb: &Block) -> &HashSet<Reg> {
        self.live_ins.get(bb.label()).unwrap()
    }
    pub fn live_outs(&self, bb: &Block) -> &HashSet<Reg> {
        self.live_outs.get(bb.label()).unwrap()
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
