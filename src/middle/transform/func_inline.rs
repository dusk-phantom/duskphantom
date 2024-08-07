use std::collections::HashMap;

use anyhow::Result;

use crate::{
    backend::from_self::downcast_ref,
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

pub fn optimize_program(program: &mut Program) -> Result<()> {
    FuncInline::new(program).run();
    Ok(())
}

struct FuncInline<'a> {
    program: &'a mut Program,
    counter: u32,
}

impl<'a> FuncInline<'a> {
    fn new(program: &'a mut Program) -> Self {
        Self {
            program,
            counter: 0,
        }
    }

    fn run(&mut self) {
        let call_graph = CallGraph::new(self.program);
        for node in call_graph.po_iter() {
            self.process_func(node.fun);
        }
    }

    fn process_func(&mut self, fun: FunPtr) {
        // Refuse to process library function
        if fun.is_lib() {
            return;
        }

        // Collect all calls in the function
        let calls = fun
            .dfs_iter()
            .flat_map(|bb| bb.iter())
            .filter(|inst| inst.get_type() == InstType::Call)
            .collect::<Vec<_>>();

        // Process each call
        for call in calls {
            self.process_call(call, fun);
        }
    }

    fn process_call(&mut self, mut inst: InstPtr, caller: FunPtr) {
        let call = downcast_ref::<Call>(inst.as_ref().as_ref());
        let func = call.func;

        // Refuse to inline library function
        if func.is_lib() {
            return;
        }

        // If self-recursive, refuse to inline
        if func == caller {
            return;
        }

        // Build argument map
        let params = func.params.iter().cloned();
        let args = inst.get_operand().iter().cloned();
        let arg_map = params.zip(args).collect();

        // Mirror function
        let new_fun = self.mirror_func(func, arg_map);
        let mut before_entry = call.get_parent_bb().unwrap();
        let after_exit = self.split_block_at(before_entry, inst);

        // Wire before_entry -> fun_entry
        let fun_entry = new_fun.entry.unwrap();
        before_entry.push_back(self.program.mem_pool.get_br(None));
        before_entry.set_true_bb(fun_entry);

        // Replace call with operand of return, remove return
        let mut fun_exit = new_fun.exit.unwrap();
        let mut ret = fun_exit.get_last_inst();
        if inst.get_value_type() == ValueType::Void {
            inst.remove_self();
        } else {
            if ret.get_operand().is_empty() {
                panic!("inst `{}` should be valued return", ret.gen_llvm_ir());
            }
            inst.replace_self(&ret.get_operand()[0]);
        }
        ret.remove_self();

        // Wire func_exit -> after_exit
        fun_exit.push_back(self.program.mem_pool.get_br(None));
        fun_exit.set_true_bb(after_exit);
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
    fn mirror_func(&mut self, fun: FunPtr, arg_map: HashMap<ParaPtr, Operand>) -> FunPtr {
        let mut inst_map: HashMap<InstPtr, InstPtr> = HashMap::new();
        let mut block_map: HashMap<BBPtr, BBPtr> = HashMap::new();
        let mut new_fun = self
            .program
            .mem_pool
            .new_function(String::new(), fun.return_type.clone());

        // Copy blocks and instructions
        for bb in fun.dfs_iter() {
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
        new_fun.entry = block_map.get(&fun.entry.unwrap()).cloned();
        new_fun.exit = block_map.get(&fun.exit.unwrap()).cloned();

        // Copy operands from old instruction to new instruction,
        // replace operands to local instruction and inlined argument
        for bb in fun.dfs_iter() {
            for inst in bb.iter() {
                let mut new_inst = inst_map.get(&inst).cloned().unwrap();
                if inst.get_type() == InstType::Phi {
                    let inst = downcast_ref::<Phi>(inst.as_ref().as_ref());
                    let new_inst = downcast_mut::<Phi>(new_inst.as_mut().as_mut());
                    for (old_op, old_bb) in inst.get_incoming_values().iter() {
                        let new_bb = block_map.get(old_bb).cloned().unwrap();
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
        for bb in fun.dfs_iter() {
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
        new_fun
    }

    fn unique_name(&mut self, meta: &str, base_name: &str) -> String {
        let name = format!("{}_{}{}", base_name, meta, self.counter);
        self.counter += 1;
        name
    }
}
