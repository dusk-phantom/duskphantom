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

use super::{gen_asm::GenTool, *};

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Block {
    label: String,
    insts: Vec<Inst>,
    pub depth: usize,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            label: "default".to_string(),
            insts: vec![],
            depth: 0,
        }
    }
}

impl Block {
    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    pub fn set_label(&mut self, label: &str) {
        self.label = label.to_string();
    }

    pub fn new(label: String) -> Block {
        Block {
            label,
            ..Default::default()
        }
    }

    pub fn push_inst(&mut self, inst: Inst) {
        self.insts.push(inst);
    }

    pub fn extend_insts(&mut self, insts: Vec<Inst>) {
        self.insts.extend(insts);
    }

    pub fn gen_asm(&self) -> String {
        let label = self.label.as_str();

        let insts = self
            .insts
            .iter()
            .map(|inst| inst.gen_asm())
            .collect::<Vec<String>>()
            .join("\n");
        GenTool::gen_bb(label, insts.as_str())
    }

    pub fn insts(&self) -> &Vec<Inst> {
        &self.insts
    }

    pub fn insts_mut(&mut self) -> &mut Vec<Inst> {
        &mut self.insts
    }

    pub fn insert_before_term(&mut self, inst: Inst) -> Result<()> {
        let is_term = |inst: &Inst| {
            matches!(
                inst,
                Inst::Jmp(_)
                    | Inst::Tail(_)
                    | Inst::Ret
                    | Inst::Beq(_)
                    | Inst::Bne(_)
                    | Inst::Blt(_)
                    | Inst::Ble(_)
                    | Inst::Bgt(_)
                    | Inst::Bge(_)
            )
        };
        let first_term = self.insts.iter().position(is_term);
        if let Some(first_term) = first_term {
            self.insts.insert(first_term, inst);
        } else {
            unreachable!("no term inst in block");
        }
        Ok(())
    }
}

impl Block {
    /// this func is only available while
    /// self's last inst is a jmp to other block
    /// this func will merge other block to self
    /// note: this func is specially designed for use in Func::merge_bb
    pub fn merge(&mut self, other: Block) -> Result<()> {
        let last = self.insts.pop();
        if let Some(Inst::Jmp(jmp)) = last {
            let dst = jmp.dst();
            if let Operand::Label(label) = dst {
                if label.as_str() != other.label() {
                    return Err(anyhow!("can't merge block"));
                }
            }
        }
        self.insts.extend(other.insts);
        Ok(())
    }
}
