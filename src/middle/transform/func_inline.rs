use std::collections::HashMap;

use anyhow::Result;

use crate::{
    backend::from_self::downcast_ref,
    middle::{
        ir::{
            instruction::{misc_inst::Call, terminator_inst::Ret, InstType},
            BBPtr, FunPtr, InstPtr, Instruction, Operand, ParaPtr,
        },
        Program,
    },
};

struct FuncInline<'a> {
    program: &'a mut Program,
    counter: u32,
}

impl<'a> FuncInline<'a> {
    fn unique_name(&mut self, base_name: String) -> String {
        let name = format!("inline_{}_{}", base_name, self.counter);
        self.counter += 1;
        name
    }

    fn process_func(&mut self, fun: FunPtr) {
        let calls = fun
            .dfs_iter()
            .flat_map(|bb| bb.iter())
            .filter(|inst| inst.get_type() == InstType::Call)
            .collect::<Vec<_>>();
        for call in calls {
            self.process_call(call);
        }
    }

    fn process_call(&mut self, mut inst: InstPtr) {
        let call = downcast_ref::<Call>(inst.as_ref().as_ref());
        let func = call.func;

        // Build argument map
        let params = func.params.iter().cloned();
        let args = inst.get_operand().iter().cloned();
        let arg_map = params.zip(args).collect();

        // Mirror function
        let new_func = self.mirror_func(func, arg_map);
        let mut before_entry = call.get_parent_bb().unwrap();
        let after_exit = self.split_block_at(before_entry, inst);

        // Wire before_entry -> func_entry
        before_entry.push_back(self.program.mem_pool.get_br(None));
        before_entry.set_true_bb(new_func.entry.unwrap());

        // Replace call with operand of return, remove return
        let mut ret = new_func.exit.unwrap().get_last_inst();
        inst.replace_self(&ret.get_operand()[0]);
        ret.remove_self();

        // Wire func_exit -> after_exit
        new_func
            .exit
            .unwrap()
            .push_back(self.program.mem_pool.get_br(None));
        new_func.exit.unwrap().set_true_bb(after_exit);
    }

    /// Split given basic block at the position of given instruction.
    /// Given instruction and instruction afterwards will be put to exit block.
    /// Returns exit block.
    fn split_block_at(&mut self, bb: BBPtr, inst: InstPtr) -> BBPtr {
        let name = self.unique_name(bb.name.clone());
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

        // Return created block
        new_bb
    }

    /// Mirror a function with given mapping.
    /// The function should not be added to program, please wire entry and exit to existing function.
    fn mirror_func(&mut self, fun: FunPtr, arg_map: HashMap<ParaPtr, Operand>) -> FunPtr {
        let mut inst_map: HashMap<InstPtr, InstPtr> = HashMap::new();
        let mut block_map: HashMap<BBPtr, BBPtr> = HashMap::new();
        let mut new_func = self
            .program
            .mem_pool
            .new_function(String::new(), fun.return_type.clone());

        // Copy blocks and instructions
        for bb in fun.dfs_iter() {
            let name = self.unique_name(bb.name.clone());
            let mut new_bb = self.program.mem_pool.new_basicblock(name);
            block_map.insert(bb, new_bb);
            for inst in bb.iter() {
                let new_inst = self
                    .program
                    .mem_pool
                    .copy_instruction(inst.as_ref().as_ref());
                inst_map.insert(inst, new_inst);
                new_bb.push_back(inst);
            }
        }

        // Set entry and exit for new function
        new_func.entry = block_map.get(&fun.entry.unwrap()).cloned();
        new_func.exit = block_map.get(&fun.exit.unwrap()).cloned();

        // Replace operand to local instruction and inlined argument
        // Safety: new and old instruction set does not overlap, so set_operand is safe
        for bb in new_func.dfs_iter() {
            for inst in bb.iter() {
                for (ix, op) in inst.get_operand().iter().enumerate() {
                    if let Operand::Instruction(old_inst) = op {
                        let new_inst = inst_map.get(old_inst).cloned().unwrap();
                        inst.clone().set_operand(ix, new_inst.into());
                    } else if let Operand::Parameter(old_param) = op {
                        let new_op = arg_map.get(old_param).cloned().unwrap();
                        inst.clone().set_operand(ix, new_op);
                    }
                }
            }
        }

        // Return new function
        new_func
    }
}
