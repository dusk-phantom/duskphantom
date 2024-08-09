use anyhow::Result;

use crate::middle::{
    analysis::memory_ssa::{MemorySSA, Node},
    ir::InstPtr,
    Program,
};

pub fn optimize_program<'a>(
    program: &'a mut Program,
    memory_ssa: &'a mut MemorySSA<'a>,
) -> Result<()> {
    StoreElim::new(program, memory_ssa).run();
    Ok(())
}

struct StoreElim<'a> {
    program: &'a mut Program,
    memory_ssa: &'a mut MemorySSA<'a>,
}

impl<'a> StoreElim<'a> {
    fn new(program: &'a mut Program, memory_ssa: &'a mut MemorySSA<'a>) -> Self {
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
            for bb in func.po_iter() {
                for inst in bb.iter() {
                    self.process_inst(inst);
                }
            }
        }
    }

    fn process_inst(&mut self, load_inst: InstPtr) {
        // Get corresponding MemorySSA node
        let Some(node) = self.memory_ssa.get_inst_node(load_inst) else {
            return;
        };

        // It should be a normal node (not entry or phi)
        let Node::Normal(_, _, _, _, _) = node.as_ref() else {
            return;
        };

        // Remove the node if it's unused
        self.memory_ssa.remove_node_recurse(node);
    }
}
