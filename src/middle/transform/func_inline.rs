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

use std::collections::HashMap;

use crate::middle::ir::instruction::downcast_ref;
use crate::{
    context,
    middle::{
        analysis::call_graph::{CallEdge, CallGraph},
        ir::{
            instruction::{
                downcast_mut,
                misc_inst::{Call, Phi},
                InstType,
            },
            BBPtr, FunPtr, InstPtr, Instruction, Operand, ParaPtr, ValueType,
        },
        Program,
    },
};
use anyhow::{anyhow, Context, Result};
use duskphantom_utils::paral_counter::ParalCounter;

use super::Transform;

pub fn optimize_program(program: &mut Program) -> Result<bool> {
    let mut call_graph = CallGraph::new(program);
    let counter = ParalCounter::new(0, usize::MAX);
    let mut func_inline = FuncInline::new(program, &mut call_graph, counter);
    func_inline.run_and_log()
}

pub struct FuncInline<'a> {
    program: &'a mut Program,
    call_graph: &'a mut CallGraph,
    counter: ParalCounter,
}

impl<'a> Transform for FuncInline<'a> {
    fn get_program_mut(&mut self) -> &mut Program {
        self.program
    }

    fn name() -> String {
        "func_inline".to_string()
    }

    fn run(&mut self) -> Result<bool> {
        let mut whole_changed = false;
        loop {
            let mut changed = false;
            for func in self.program.module.functions.clone() {
                // Do not process library function
                if func.is_lib() {
                    continue;
                }

                // If functions calls other functions, do not process it
                if !self.call_graph.get_calls(func).is_empty() {
                    continue;
                }

                // Process function
                changed |= self.process_func(func)?;
                whole_changed |= changed;

                // Update call graph
                self.call_graph.remove(func);
            }
            if !changed {
                break;
            }
        }
        Ok(whole_changed)
    }
}

impl<'a> FuncInline<'a> {
    pub fn new(
        program: &'a mut Program,
        call_graph: &'a mut CallGraph,
        counter: ParalCounter,
    ) -> Self {
        Self {
            program,
            call_graph,
            counter,
        }
    }

    fn process_func(&mut self, func: FunPtr) -> Result<bool> {
        let mut changed = false;

        // Eliminate call to func
        for call in self.call_graph.get_called_by(func) {
            changed |= self.process_call(call)?;
        }

        // Delete func to reduce code size
        if changed {
            self.program.module.functions.retain(|&f| f != func);
        }
        Ok(changed)
    }

    fn process_call(&mut self, edge: CallEdge) -> Result<bool> {
        let mut inst = edge.inst;
        let call = downcast_ref::<Call>(inst.as_ref().as_ref());

        // Build argument map
        let params = edge.callee.params.iter().cloned();
        let args = inst.get_operand().iter().cloned();
        let arg_map = params.zip(args).collect();

        // Mirror function, focus on interface basic blocks
        let new_fun = self.mirror_func(edge.callee, arg_map)?;
        let mut before_entry = call.get_parent_bb().unwrap();
        let after_exit = self.split_block_at(before_entry, inst)?;
        let fun_entry = new_fun
            .entry
            .ok_or_else(|| anyhow!("function `{}` has no entry", new_fun.name))
            .with_context(|| context!())?;
        let mut fun_exit = new_fun
            .exit
            .ok_or_else(|| anyhow!("function `{}` has no exit", new_fun.name))
            .with_context(|| context!())?;

        // Wire before_entry -> fun_entry
        before_entry.push_back(self.program.mem_pool.get_br(None));
        before_entry.set_true_bb(fun_entry);

        // Replace call with operand of return, remove return
        let mut ret = fun_exit.get_last_inst();
        if inst.get_value_type() == ValueType::Void {
            inst.remove_self();
        } else {
            let ret_val = ret
                .get_operand()
                .first()
                .ok_or_else(|| anyhow!("function `{}` has no return value", new_fun.name))
                .with_context(|| context!())?;
            inst.replace_self(ret_val);
        }
        ret.remove_self();

        // Wire func_exit -> after_exit
        fun_exit.push_back(self.program.mem_pool.get_br(None));
        fun_exit.set_true_bb(after_exit);
        Ok(true)
    }

    /// Split given basic block at the position of given instruction.
    /// Given instruction and instruction afterwards will be put to exit block.
    /// Returns new exit block.
    fn split_block_at(&mut self, mut entry: BBPtr, inst: InstPtr) -> Result<BBPtr> {
        let exit_name = self.unique_name("split", &entry.name);
        let mut exit = self.program.mem_pool.new_basicblock(exit_name);
        let mut split = false;

        // Copy instructions after found target instruction
        for bb_inst in entry.iter() {
            if bb_inst == inst {
                split = true;
            }
            if split {
                exit.push_back(bb_inst);
            }
        }

        // Replace `entry` with `entry -> exit`
        entry.replace_exit(exit);

        // Return created block
        Ok(exit)
    }

    /// Mirror a function with given mapping.
    /// The function should not be added to program, please wire entry and exit to existing function.
    fn mirror_func(&mut self, func: FunPtr, arg_map: HashMap<ParaPtr, Operand>) -> Result<FunPtr> {
        let func_entry = func
            .entry
            .ok_or_else(|| anyhow!("function `{}` has no entry", func.name))
            .with_context(|| context!())?;
        let func_exit = func
            .exit
            .ok_or_else(|| anyhow!("function `{}` has no exit", func.name))
            .with_context(|| context!())?;

        // Initialize inst and block mapping and new function
        let mut inst_map: HashMap<InstPtr, InstPtr> = HashMap::new();
        let mut block_map: HashMap<BBPtr, BBPtr> = HashMap::new();
        let mut new_fun = self
            .program
            .mem_pool
            .new_function(String::new(), func.return_type.clone());

        // Copy blocks and instructions
        for bb in func.dfs_iter() {
            let name = self.unique_name("inline", &bb.name);
            let mut new_bb = self.program.mem_pool.new_basicblock(name);
            block_map.insert(bb, new_bb);
            for inst in bb.iter() {
                let new_inst = self
                    .program
                    .mem_pool
                    .copy_instruction(inst.as_ref().as_ref());
                inst_map.insert(inst, new_inst);
                new_bb.push_back(new_inst);
            }
        }

        // Set entry and exit for new function
        new_fun.entry = block_map.get(&func_entry).cloned();
        new_fun.exit = block_map.get(&func_exit).cloned();

        // Copy operands from old instruction to new instruction,
        // replace operands to local instruction and inlined argument
        for bb in func.dfs_iter() {
            for inst in bb.iter() {
                let mut new_inst = inst_map
                    .get(&inst)
                    .cloned()
                    .ok_or_else(|| anyhow!("instruction not found in inst_map: {}", inst))
                    .with_context(|| context!())?;
                if inst.get_type() == InstType::Phi {
                    let inst = downcast_ref::<Phi>(inst.as_ref().as_ref());
                    let new_inst = downcast_mut::<Phi>(new_inst.as_mut());

                    // Replace operand for phi instruction
                    for (old_op, old_bb) in inst.get_incoming_values().iter() {
                        let new_bb = block_map
                            .get(old_bb)
                            .cloned()
                            .ok_or_else(|| anyhow!("bb not found in block_map: {}", old_bb.name))
                            .with_context(|| context!())?;
                        if let Operand::Instruction(old_op) = old_op {
                            let new_op = inst_map.get(old_op).cloned().unwrap();
                            new_inst.add_incoming_value(new_op.into(), new_bb);
                        } else if let Operand::Parameter(old_op) = old_op {
                            let new_op = arg_map.get(old_op).cloned().unwrap();
                            new_inst.add_incoming_value(new_op, new_bb);
                        } else {
                            // Copy operands manually because `copy_instruction` does not copy them
                            new_inst.add_incoming_value(old_op.clone(), new_bb);
                        }
                    }
                } else {
                    // Replace operand for normal instruction
                    for old_op in inst.get_operand().iter() {
                        if let Operand::Instruction(old_op) = old_op {
                            let new_op = inst_map.get(old_op).cloned().unwrap();
                            new_inst.add_operand(new_op.into());
                        } else if let Operand::Parameter(old_op) = old_op {
                            let new_op = arg_map.get(old_op).cloned().unwrap();
                            new_inst.add_operand(new_op);
                        } else {
                            // Copy operands manually because `copy_instruction` does not copy them
                            new_inst.add_operand(old_op.clone());
                        }
                    }
                }
            }
        }

        // Assign mapped basic blocks to successor
        for bb in func.dfs_iter() {
            let mut new_bb = block_map.get(&bb).cloned().unwrap();
            let succ_bb = bb.get_succ_bb();
            if !succ_bb.is_empty() {
                let new_succ = block_map.get(&succ_bb[0]).cloned().unwrap();
                new_bb.set_true_bb(new_succ);
            }
            if succ_bb.len() >= 2 {
                let new_succ = block_map.get(&succ_bb[1]).cloned().unwrap();
                new_bb.set_false_bb(new_succ);
            }
        }

        // Return new function
        Ok(new_fun)
    }

    fn unique_name(&mut self, meta: &str, base_name: &str) -> String {
        format!("{}_{}{}", base_name, meta, self.counter.get_id().unwrap())
    }
}
