use anyhow::Result;

use crate::middle::{
    analysis::memory_ssa::{MemorySSA, Node},
    ir::{instruction::InstType, InstPtr},
    Program,
};

pub fn optimize_program<'a>(program: &'a mut Program, memory_ssa: &'a mut MemorySSA) -> Result<()> {
    LoadElim::new(program, memory_ssa).run();
    Ok(())
}

struct LoadElim<'a, 'b> {
    program: &'a mut Program,
    memory_ssa: &'a mut MemorySSA<'b>,
}

impl<'a, 'b> LoadElim<'a, 'b> {
    fn new(program: &'a mut Program, memory_ssa: &'a mut MemorySSA<'b>) -> Self {
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
            for bb in func.rpo_iter() {
                for inst in bb.iter() {
                    self.process_inst(inst);
                }
            }
        }
    }

    fn process_inst(&mut self, mut load_inst: InstPtr) {
        // Instruction must be load (instead of function call), otherwise it can't be optimized
        if load_inst.get_type() != InstType::Load {
            return;
        }

        // Get corresponding MemorySSA node
        let Some(load_node) = self.memory_ssa.get_inst_node(load_inst) else {
            return;
        };

        // It should be a linear normal node (not entry or phi)
        // Linear node means this MemoryUse is predictable
        // (when `a[1] = 3`, `load a[x]` is not linear because `x` may or may not be `1`, but `load a[1]` is linear)
        let Node::Normal(_, used_node, _, _, true) = load_node.as_ref() else {
            return;
        };

        // MemoryUse should use some node
        let Some(store_node) = used_node else {
            return;
        };

        // The node used by MemoryUse should be a MemoryDef
        let Node::Normal(_, _, _, store_inst, _) = store_node.as_ref() else {
            return;
        };

        // The MemoryDef should be store (instead of function call), otherwise it can't be optimized
        if store_inst.get_type() != InstType::Store {
            return;
        }

        // Replace load with operand of store
        let store_op = store_inst.get_operand().first().unwrap();
        load_inst.replace_self(store_op);
        self.memory_ssa.remove_node(load_node);
    }
}
