use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};

use crate::{
    backend::from_self::downcast_ref,
    context,
    middle::{
        analysis::call_graph::CallGraph,
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

pub fn optimize_program(program: &mut Program, call_graph: &mut CallGraph) -> Result<bool> {
    FuncInline::new(program, call_graph).run()
}

struct FuncInline<'a> {
    program: &'a mut Program,
    call_graph: &'a mut CallGraph,
    counter: u32,
}

impl<'a> FuncInline<'a> {
    fn new(program: &'a mut Program, call_graph: &'a mut CallGraph) -> Self {
        Self {
            program,
            call_graph,
            counter: 0,
        }
    }

    fn run(&mut self) -> Result<bool> {
        let mut overall_changed = false;
        let mut changed = false;
        while changed {
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
                overall_changed |= changed;

                // Update call graph
                self.call_graph.remove(func);
            }
        }
        Ok(overall_changed)
    }

    fn process_func(&mut self, fun: FunPtr) -> Result<bool> {
        let mut changed = false;
        for call in self.call_graph.get_called_by(fun) {
            changed |= self.process_call(call.inst)?;
        }
        Ok(changed)
    }

    fn process_call(&mut self, mut inst: InstPtr) -> Result<bool> {
        let call = downcast_ref::<Call>(inst.as_ref().as_ref());
        let func = call.func;

        // Build argument map
        let params = func.params.iter().cloned();
        let args = inst.get_operand().iter().cloned();
        let arg_map = params.zip(args).collect();

        // Mirror function, focus on interface basic blocks
        let new_fun = self.mirror_func(func, arg_map)?;
        let mut before_entry = call.get_parent_bb().unwrap();
        let after_exit = self.split_block_at(before_entry, inst);
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
    /// Argument block will be entry block, returns exit block.
    fn split_block_at(&mut self, bb: BBPtr, inst: InstPtr) -> BBPtr {
        let name = self.unique_name("split", &bb.name);
        let mut new_bb = self.program.mem_pool.new_basicblock(name);
        let mut split = false;

        // Copy instructions after found target instruction
        for bb_inst in bb.iter() {
            if bb_inst == inst {
                split = true;
            }
            if split {
                new_bb.push_back(bb_inst);
            }
        }

        // Copy jump destination from bb, handle phi argument changes
        if !bb.get_succ_bb().is_empty() {
            let succ = bb.get_succ_bb()[0];
            new_bb.set_true_bb(succ);
            self.replace_bb_in_phi(succ, bb, new_bb);
        }
        if bb.get_succ_bb().len() >= 2 {
            let succ = bb.get_succ_bb()[1];
            new_bb.set_false_bb(succ);
            self.replace_bb_in_phi(succ, bb, new_bb);
        }

        // Return created block
        new_bb
    }

    /// Replace basic block in phi instruction with given mapping.
    fn replace_bb_in_phi(&mut self, bb: BBPtr, from: BBPtr, to: BBPtr) {
        for mut inst in bb.iter() {
            if inst.get_type() == InstType::Phi {
                let inst = downcast_mut::<Phi>(inst.as_mut().as_mut());
                for (_, bb) in inst.get_incoming_values_mut().iter_mut() {
                    if *bb == from {
                        *bb = to;
                    }
                }
            }
        }
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
                    let new_inst = downcast_mut::<Phi>(new_inst.as_mut().as_mut());

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

        // Replace succ bb
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
        let name = format!("{}_{}{}", base_name, meta, self.counter);
        self.counter += 1;
        name
    }
}
