use anyhow::Result;

use crate::middle::{
    analysis::memory_ssa::{MemorySSA, NodePtr},
    ir::{instruction::InstType, InstPtr, Operand},
    Program,
};

use super::Transform;

pub fn optimize_program<'a>(
    program: &'a mut Program,
    memory_ssa: &'a mut MemorySSA<'a>,
) -> Result<bool> {
    StoreElim::new(program, memory_ssa).run_and_log()
}

pub struct StoreElim<'a> {
    program: &'a mut Program,
    memory_ssa: &'a mut MemorySSA<'a>,
}

impl<'a> Transform for StoreElim<'a> {
    fn name() -> String {
        "store_elim".to_string()
    }

    fn run(&mut self) -> Result<bool> {
        let mut changed = false;
        for func in self.program.module.functions.clone() {
            if func.is_main() {
                for bb in func.po_iter() {
                    for inst in bb.iter() {
                        changed |= self.process_inst(inst)?;
                    }
                }
            }
        }
        Ok(changed)
    }
}

impl<'a> StoreElim<'a> {
    pub fn new(program: &'a mut Program, memory_ssa: &'a mut MemorySSA<'a>) -> Self {
        Self {
            program,
            memory_ssa,
        }
    }

    /// Delete instruction and its corresponding MemorySSA node if it's not used.
    /// This recurses into operands of the instruction.
    fn process_inst(&mut self, inst: InstPtr) -> Result<bool> {
        if !self.can_delete_inst(inst) {
            return Ok(false);
        }
        if let Some(node) = self.memory_ssa.get_inst_node(inst) {
            if !self.can_delete_node(node) {
                return Ok(false);
            }
            self.remove_node(node)?;
        };
        self.remove_inst(inst)?;
        Ok(true)
    }

    /// Delete MemorySSA node if unused.
    /// This recurses into used nodes of the node.
    fn process_node(&mut self, node: NodePtr) -> Result<bool> {
        if !self.can_delete_node(node) {
            return Ok(false);
        }
        if let Some(inst) = node.get_inst() {
            if !self.can_delete_inst(inst) {
                return Ok(false);
            }
            self.remove_inst(inst)?;
        }
        self.remove_node(node)?;
        Ok(true)
    }

    /// Check if instruction can be deleted.
    fn can_delete_inst(&self, inst: InstPtr) -> bool {
        let no_io = !self.memory_ssa.effect_analysis.has_io(inst);
        let no_user = inst.get_user().is_empty();
        let no_control = !matches!(inst.get_type(), InstType::Br | InstType::Ret);
        no_io && no_user && no_control
    }

    /// Check if node can be deleted.
    fn can_delete_node(&self, node: NodePtr) -> bool {
        self.memory_ssa.get_user(node).is_empty()
    }

    /// Remove instruction and recurse into operands.
    fn remove_inst(&mut self, mut inst: InstPtr) -> Result<()> {
        let operands: Vec<_> = inst.get_operand().into();
        inst.remove_self();
        for op in operands {
            if let Operand::Instruction(inst) = op {
                self.process_inst(inst)?;
            }
        }
        Ok(())
    }

    /// Remove node and recurse into used nodes.
    fn remove_node(&mut self, node: NodePtr) -> Result<()> {
        let used_node = node.get_used_node();
        self.memory_ssa.remove_node(node);
        for node in used_node {
            self.process_node(node)?;
        }
        Ok(())
    }
}
