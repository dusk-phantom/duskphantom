use anyhow::Result;

use crate::middle::{
    analysis::memory_ssa::{MemorySSA, Node},
    ir::{instruction::InstType, InstPtr},
    Program,
};

pub fn optimize_program(program: &mut Program, memory_ssa: &MemorySSA) -> Result<()> {
    LoadElim::new(program, memory_ssa).run();
    Ok(())
}

struct LoadElim<'a> {
    program: &'a mut Program,
    memory_ssa: &'a MemorySSA<'a>,
}

impl<'a> LoadElim<'a> {
    fn new(program: &'a mut Program, memory_ssa: &'a MemorySSA<'a>) -> Self {
        Self {
            program,
            memory_ssa,
        }
    }

    fn run(&mut self) {
        for func in self.program.module.functions.clone().iter() {
            if func.is_lib() {
                continue;
            }
            for bb in func.dfs_iter() {
                for inst in bb.iter() {
                    self.process_inst(inst);
                }
            }
        }
    }

    fn process_inst(&mut self, mut load_inst: InstPtr) {
        // Instruction must be load
        if load_inst.get_type() != InstType::Load {
            return;
        }

        // Get corresponding MemorySSA node
        let Some(node) = self.memory_ssa.get_inst_node(load_inst) else {
            return;
        };

        // It should be a normal node (not entry or phi)
        let Node::Normal(_, use_node, _, _) = node.as_ref() else {
            return;
        };

        // MemoryUse should use some node
        let Some(node) = use_node else {
            return;
        };

        // The node used by MemoryUse should be a MemoryDef
        let Some(store_inst) = node.get_inst() else {
            return;
        };

        // The MemoryDef should be store, otherwise it can't be optimized
        if store_inst.get_type() != InstType::Store {
            return;
        }

        // Replace load with operand of store
        load_inst.replace_self(store_inst.get_operand().first().unwrap());
    }
}
