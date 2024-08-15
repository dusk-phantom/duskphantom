use super::*;
use rustc_hash::FxHashSet;
use std::fmt::{Debug, Display};

impl Block {
    pub fn live_use_regs(&self) -> FxHashSet<Reg> {
        let mut live_use = FxHashSet::default();
        for inst in self.insts().iter().rev() {
            live_use.retain(|r| !inst.defs().contains(&r));
            live_use.extend(inst.uses().iter().cloned());
        }
        live_use
    }

    pub fn live_def_regs(&self) -> FxHashSet<Reg> {
        let mut live_def = FxHashSet::default();
        for inst in self.insts().iter().rev() {
            live_def.extend(inst.defs().iter().cloned());
            live_def.retain(|r| !inst.uses().contains(&r));
        }
        live_def
    }
}

impl Func {
    pub fn reg_live_use(f: &Func) -> HashMap<String, FxHashSet<Reg>> {
        f.iter_bbs()
            .map(|bb| (bb.label().to_string(), bb.live_use_regs()))
            .collect()
    }

    pub fn reg_live_def(f: &Func) -> HashMap<String, FxHashSet<Reg>> {
        f.iter_bbs()
            .map(|bb| (bb.label().to_string(), bb.live_def_regs()))
            .collect()
    }

    /// compute the live in and live out set of regs of each basic block
    pub fn reg_lives(f: &Func) -> Result<RegLives> {
        let (ins, outs) = Func::in_out_bbs(f)?;

        let mut live_ins: HashMap<String, FxHashSet<Reg>> = HashMap::new();
        let mut live_outs: HashMap<String, FxHashSet<Reg>> = HashMap::new();

        // consider the exit block
        if let Some(ret) = f.ret() {
            for bb in f.exit_bbs() {
                let mut live_out: FxHashSet<Reg> = FxHashSet::default();
                live_out.insert(*ret);
                live_outs.insert(bb.label().to_string(), live_out);
            }
        }
        let bb_iter = f.iter_bbs();

        // consider live_use
        let live_use = Func::reg_live_use(f);
        let live_def = Func::reg_live_def(f);

        for (bb, live_use_bb) in live_use.iter() {
            live_ins.insert(bb.to_string(), live_use_bb.clone());
        }

        // loop to compute live_in and live_out
        // FIXME: 使用位图实现的寄存器记录表来加速运算过程，以及节省内存
        let mut changed = true;
        while changed {
            changed = false;
            // new_live_out = SUM(live_in[succ[bb]])
            // new_live_in = old_live_in U ( SUM(live_in[out[bb]]) - live_def[bb] ) U new_live_out
            for bb in bb_iter.clone() {
                let mut new_live_in = live_ins
                    .get(bb.label())
                    .cloned()
                    .unwrap_or(FxHashSet::default());
                for in_bb in ins.ins(bb) {
                    if let Some(out) = live_outs.get(in_bb.label()) {
                        new_live_in.extend(out.iter().cloned());
                    }
                }
                new_live_in.retain(|r| !live_def[bb.label()].contains(r));

                let mut new_live_out = live_outs
                    .get(bb.label())
                    .cloned()
                    .unwrap_or(FxHashSet::default());
                for out_bb in outs.outs(bb) {
                    if let Some(in_) = live_ins.get(out_bb.label()) {
                        new_live_out.extend(in_.iter().cloned());
                    }
                }

                new_live_in.extend(
                    new_live_out
                        .iter()
                        .cloned()
                        .filter(|r| !live_def[bb.label()].contains(r)),
                );

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
    pub fn reg_interfere_graph(f: &Func) -> Result<HashMap<Reg, FxHashSet<Reg>>> {
        fn add_inter(g: &mut HashMap<Reg, FxHashSet<Reg>>, r1: &Reg, r2: &Reg) {
            if r1.is_virtual() || r2.is_virtual() {
                if r1 == r2 {
                    g.entry(*r1).or_default();
                    return;
                }
                g.entry(*r1).or_default().insert(*r2);
                g.entry(*r2).or_default().insert(*r1);
            }
        }
        fn add_node(g: &mut HashMap<Reg, FxHashSet<Reg>>, r: &Reg) {
            if r.is_virtual() {
                g.entry(*r).or_default();
            }
        }
        // for each basic block, collect interference between regs
        let mut graph: HashMap<Reg, FxHashSet<Reg>> = HashMap::new();
        let reg_lives = Func::reg_lives(f)?;
        // dbg!(&reg_lives);
        // FIXME: 使用位图实现的寄存器记录表来加速运算过程，以及节省内存
        for bb in f.iter_bbs() {
            let mut alive_regs: FxHashSet<Reg> = reg_lives.live_outs(bb).clone();
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

pub struct RegLives {
    live_ins: HashMap<String, FxHashSet<Reg>>,
    live_outs: HashMap<String, FxHashSet<Reg>>,
}
impl Debug for RegLives {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegLives")
            .field("live_ins", {
                let mut live_ins: Vec<(&String, &FxHashSet<Reg>)> = self.live_ins.iter().collect();
                live_ins.sort_by(|(k, _), (k2, _)| k.cmp(k2));
                &live_ins
                    .into_iter()
                    .map(|(k, v)| {
                        let mut v = v.iter().map(|v| v.gen_asm()).collect::<Vec<String>>();
                        v.sort();
                        (k, v)
                    })
                    .collect::<Vec<_>>()
            })
            .field("live_outs", {
                let mut live_outs: Vec<(&String, &FxHashSet<Reg>)> =
                    self.live_outs.iter().collect();
                live_outs.sort_by(|(k, _), (k2, _)| k.cmp(k2));
                &live_outs
                    .into_iter()
                    .map(|(k, v)| {
                        let mut v = v.iter().map(|v| v.gen_asm()).collect::<Vec<String>>();
                        v.sort();
                        (k, v)
                    })
                    .collect::<Vec<_>>()
            })
            .finish()
    }
}

impl Display for RegLives {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "live_ins:")?;
        let mut live_ins: Vec<(&String, &FxHashSet<Reg>)> = self.live_ins.iter().collect();
        live_ins.sort_by(|(k, _), (k2, _)| k.cmp(k2));
        for (k, v) in live_ins {
            let mut v = v.iter().map(|v| v.gen_asm()).collect::<Vec<String>>();
            v.sort();
            writeln!(f, "{}: {:?}", k, v)?;
        }
        writeln!(f, "live_outs:")?;
        let mut live_outs: Vec<(&String, &FxHashSet<Reg>)> = self.live_outs.iter().collect();
        live_outs.sort_by(|(k, _), (k2, _)| k.cmp(k2));
        for (k, v) in live_outs {
            let mut v = v.iter().map(|v| v.gen_asm()).collect::<Vec<String>>();
            v.sort();
            writeln!(f, "{}: {:?}", k, v)?;
        }
        Ok(())
    }
}

impl RegLives {
    pub fn live_ins(&self, bb: &Block) -> &FxHashSet<Reg> {
        self.live_ins.get(bb.label()).unwrap()
    }
    pub fn live_outs(&self, bb: &Block) -> &FxHashSet<Reg> {
        self.live_outs.get(bb.label()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use rustc_hash::FxHashSet;

    use super::{AddInst, Block, MvInst, Reg};
    fn stringfy_regs(regs: &FxHashSet<Reg>) -> String {
        let mut regs: Vec<String> = regs.iter().map(|r| r.gen_asm()).collect();
        regs.sort();
        format!("{{{}}}", regs.join(","))
    }

    #[test]
    fn test_bb_live_use() {
        // addiw x42,x38,1
        // mv x32,x42
        // mv x33,x35
        // j .Lmain_cond0
        let x38 = Reg::new(38, true);
        let x32 = Reg::new(32, true);
        let x42 = Reg::new(42, true);
        let x35 = Reg::new(35, true);
        let x33 = Reg::new(33, true);
        let mut bb = Block::new("".to_string());
        bb.push_inst(AddInst::new(x42.into(), x38.into(), 1.into()).into());
        bb.push_inst(MvInst::new(x32.into(), x42.into()).into());
        bb.push_inst(MvInst::new(x33.into(), x35.into()).into());
        let live_use = bb.live_use_regs();
        assert_snapshot!(stringfy_regs(&live_use),@"{x35,x38}");
    }
}
