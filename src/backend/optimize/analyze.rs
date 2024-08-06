use std::fmt::Debug;

use super::*;

impl Block {
    pub fn to_bbs(bb: &Block) -> Result<Vec<String>> {
        let mut tos = vec![];
        for inst in bb.insts() {
            match inst {
                Inst::Bne(b) => tos.push(b.label().to_string()),
                Inst::Blt(b) => tos.push(b.label().to_string()),
                Inst::Ble(b) => tos.push(b.label().to_string()),
                Inst::Bgt(b) => tos.push(b.label().to_string()),
                Inst::Bge(b) => tos.push(b.label().to_string()),
                Inst::Beq(b) => tos.push(b.label().to_string()),
                Inst::Jmp(jmp) => {
                    let label: &Label = jmp.dst().try_into()?;
                    tos.push(label.to_string());
                    break;
                }
                Inst::Tail(_) => break,
                Inst::Ret => break,
                _ => continue,
            }
        }
        Ok(tos)
    }
}

/// impl Some functionality for reg alloc
impl Func {
    /// compute the in and out set of each basic block
    /// return a tuple of two hashmaps, the first one is in set, the second one is out set
    pub fn in_out_bbs(f: &Func) -> Result<(InBBs, OutBBs)> {
        let mut outs: HashMap<String, Vec<String>> = HashMap::new();
        for bb in f.iter_bbs() {
            let to_bbs: Vec<String> = Block::to_bbs(bb)?;
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
        fn add_inter(g: &mut HashMap<Reg, HashSet<Reg>>, r1: &Reg, r2: &Reg) {
            if r1.is_virtual() || r2.is_virtual() {
                if r1 == r2 {
                    g.entry(*r1).or_default();
                    return;
                }
                g.entry(*r1).or_default().insert(*r2);
                g.entry(*r2).or_default().insert(*r1);
            }
        }
        fn add_node(g: &mut HashMap<Reg, HashSet<Reg>>, r: &Reg) {
            if r.is_virtual() {
                g.entry(*r).or_default();
            }
        }
        // for each basic block, collect interference between regs
        let (ins, outs) = Func::in_out_bbs(f)?;

        let reg_lives = Func::reg_lives(f, &ins, &outs)?;
        dbg!(&reg_lives);
        // FIXME: 使用位图实现的寄存器记录表来加速运算过程，以及节省内存
        for bb in f.iter_bbs() {
            let mut alive_regs: HashSet<Reg> = reg_lives.live_outs(bb).clone();
            for r in &alive_regs {
                add_node(&mut graph, r);
            }
            for inst in bb.insts().iter().rev() {
                // 计算该指令处的冲突
                // case 1: 该指令定义的寄存器与当前存活的自己以外的所有寄存器冲突
                for r in inst.defs() {
                    add_node(&mut graph, r);
                    for alive_reg in alive_regs.iter() {
                        add_inter(&mut graph, r, alive_reg);
                    }
                }
                // case 2: 该指令处使用的寄存器与(alive_regs - defs)中自己以外的所有寄存器冲突
                for r in inst.uses().iter().filter(|r1| !inst.defs().contains(r1)) {
                    add_node(&mut graph, r);
                    for alive_reg in alive_regs.iter() {
                        add_inter(&mut graph, r, alive_reg);
                    }
                }
                // 然后更新存活寄存器集合 new_alive=alive_regs-defs+uses
                alive_regs.retain(|r| !inst.defs().contains(&r));
                alive_regs.extend(inst.uses().iter().cloned());

                // dbg!(g2txt(&graph));
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
impl Debug for RegLives {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegLives")
            .field(
                "live_ins",
                &self
                    .live_ins
                    .iter()
                    .map(|(k, v)| (k, v.iter().map(|v| v.gen_asm())))
                    .collect::<Vec<_>>(),
            )
            .field(
                "live_outs",
                &self
                    .live_outs
                    .iter()
                    .map(|(k, v)| (k, v.iter().map(|v| v.gen_asm())))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}
impl RegLives {
    pub fn live_ins(&self, bb: &Block) -> &HashSet<Reg> {
        self.live_ins.get(bb.label()).unwrap()
    }
    pub fn live_outs(&self, bb: &Block) -> &HashSet<Reg> {
        self.live_outs.get(bb.label()).unwrap()
    }
}
