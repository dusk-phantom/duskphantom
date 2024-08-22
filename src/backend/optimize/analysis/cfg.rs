// Copyright 2024 Duskphantom Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

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
