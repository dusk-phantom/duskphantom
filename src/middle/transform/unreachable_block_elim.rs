use anyhow::{anyhow, Context, Result};

use crate::{
    context,
    middle::{
        analysis::reachability::Reachability,
        ir::{instruction::InstType, Constant, FunPtr, Operand},
        Program,
    },
};

use super::Transform;

pub fn optimize_program(program: &mut Program) -> Result<bool> {
    UnreachableBlockElim::new(program).run_and_log()
}

pub struct UnreachableBlockElim<'a> {
    program: &'a mut Program,
}

impl<'a> Transform for UnreachableBlockElim<'a> {
    fn name() -> String {
        "unreachable_block_elim".to_string()
    }

    fn run(&mut self) -> Result<bool> {
        let mut changed = false;
        for func in self
            .program
            .module
            .functions
            .clone()
            .iter()
            .filter(|f| !f.is_lib())
        {
            changed |= self.process_function(*func)?;
        }
        Ok(changed)
    }
}

impl<'a> UnreachableBlockElim<'a> {
    pub fn new(program: &'a mut Program) -> Self {
        Self { program }
    }

    fn process_function(&mut self, func: FunPtr) -> Result<bool> {
        let mut changed = false;

        // Simplify "br" with constant condition to unconditional
        let bbs: Vec<_> = func.dfs_iter().collect();
        for bb in bbs.iter() {
            for mut inst in bb.iter() {
                if inst.get_type() == InstType::Br {
                    let cond = inst.get_operand().first().cloned();
                    if let Some(Operand::Constant(Constant::Bool(cond))) = cond {
                        // Rewire basic block
                        let mut parent_bb = inst
                            .get_parent_bb()
                            .ok_or_else(|| anyhow!("{} should have parent block", inst))
                            .with_context(|| context!())?;
                        if cond {
                            parent_bb.remove_false_bb();
                        } else {
                            parent_bb.remove_true_bb();
                        }

                        // Replace instruction with unconditional jump
                        let new_inst = self.program.mem_pool.get_br(None);
                        inst.insert_after(new_inst);
                        inst.remove_self();
                        changed = true;
                    }
                }
            }
        }

        // Remove unreachable block
        let reachability = Reachability::new(func);
        for bb in func.dfs_iter() {
            for mut pred in bb.get_pred_bb().clone() {
                if !reachability.is_reachable(pred) {
                    pred.remove_self();
                    changed = true;
                }
            }
        }
        Ok(changed)
    }
}
