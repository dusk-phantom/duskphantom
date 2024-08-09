use anyhow::Result;

use crate::{
    backend::from_self::downcast_ref,
    middle::{
        analysis::{
            effect_analysis::EffectAnalysis,
            memory_ssa::{MemorySSA, Node},
        },
        ir::{
            instruction::{misc_inst::Call, InstType},
            InstPtr, Operand,
        },
        Program,
    },
};

pub fn optimize_program<'a>(
    program: &'a mut Program,
    memory_ssa: &'a mut MemorySSA<'a>,
) -> Result<()> {
    LoadElim::new(program, memory_ssa).run();
    Ok(())
}

struct LoadElim<'a> {
    program: &'a mut Program,
    memory_ssa: &'a mut MemorySSA<'a>,
}

impl<'a> LoadElim<'a> {
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
            for bb in func.rpo_iter() {
                for inst in bb.iter() {
                    self.process_inst(inst);
                }
            }
        }
    }

    fn process_inst(&mut self, load_inst: InstPtr) {
        // Instruction must be load (instead of function call), otherwise it can't be optimized
        if load_inst.get_type() != InstType::Load {
            return;
        }

        // Get corresponding MemorySSA node
        let Some(load_node) = self.memory_ssa.get_inst_node(load_inst) else {
            return;
        };

        // It should be a linear normal node (not entry or phi)
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
        self.memory_ssa.replace_node(load_node, store_op);
    }
}
