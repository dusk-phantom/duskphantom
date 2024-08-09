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
    pub fn live_use_regs(&self) -> HashSet<Reg> {
        let mut live_use = HashSet::new();
        for inst in self.insts().iter().rev() {
            live_use.retain(|r| !inst.defs().contains(&r));
            live_use.extend(inst.uses().iter().cloned());
        }
        live_use
    }
    pub fn live_def_regs(&self) -> HashSet<Reg> {
        let mut live_def = HashSet::new();
        for inst in self.insts().iter().rev() {
            live_def.extend(inst.defs().iter().cloned());
            live_def.retain(|r| !inst.uses().contains(&r));
        }
        live_def
    }

    pub fn ordered_insts_text(&self) -> String {
        self.insts()
            .iter()
            .enumerate()
            .map(|(i, inst)| format!("{}:\n{}", i, inst.gen_asm()))
            .fold(String::new(), |acc, x| acc + &x)
    }
}

/// impl Some functionality for reg alloc
impl Func {
    pub fn bbs_graph_to_dot(&self) -> String {
        let successors = self.successors().unwrap();
        let mut dot = String::new();
        dot.push_str("digraph bbs{\n");
        for (from, tos) in successors {
            for to in tos {
                dot.push_str(&format!("\"{}\" -> \"{}\";\n", from, to));
            }
        }
        dot.push_str("}\n");
        dot
    }

    /// return a hashmap of basic block label to its successors
    pub fn successors(&self) -> Result<HashMap<String, HashSet<String>>> {
        let mut hmp: HashMap<String, HashSet<String>> = HashMap::new();
        for bb in self.iter_bbs() {
            let to_bbs: HashSet<String> = Block::to_bbs(bb)?.into_iter().collect();
            hmp.insert(bb.label().to_string(), to_bbs);
        }
        Ok(hmp)
    }

    /// return a hashmap of basic block label to its predecessors
    pub fn predecessors_from_succs(
        &self,
        succs: &HashMap<String, HashSet<String>>,
    ) -> HashMap<String, HashSet<String>> {
        let mut hmp: HashMap<String, HashSet<String>> = HashMap::new();
        for bb in self.iter_bbs() {
            hmp.insert(bb.label().to_string(), HashSet::new());
        }
        for (from, tos) in succs {
            for to in tos {
                hmp.entry(to.to_string())
                    .or_default()
                    .insert(from.to_string());
            }
        }
        hmp
    }

    /// compute the in and out set of each basic block
    /// return a tuple of two hashmaps, the first one is in set, the second one is out set
    pub fn in_out_bbs(f: &Func) -> Result<(InBBs, OutBBs)> {
        let successors = f.successors()?;
        let predecessors = f.predecessors_from_succs(&successors);

        let bbs: HashMap<String, &Block> = f
            .iter_bbs()
            .map(|bb| (bb.label().to_string(), bb))
            .collect();
        let ins = InBBs {
            bbs: bbs.clone(),
            ins: predecessors,
        };
        let outs = OutBBs {
            bbs,
            outs: successors,
        };

        Ok((ins, outs))
    }
}

#[derive(Debug)]
pub struct InBBs<'a> {
    bbs: HashMap<String, &'a Block>,
    ins: HashMap<String, HashSet<String>>,
}
#[derive(Debug)]
pub struct OutBBs<'a> {
    bbs: HashMap<String, &'a Block>,
    outs: HashMap<String, HashSet<String>>,
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