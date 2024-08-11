use anyhow::{anyhow, Context, Result};

use crate::{
    context,
    middle::{
        analysis::reachability::Reachability,
        ir::{instruction::InstType, Constant, FunPtr, Operand},
        Program,
    },
};

pub fn optimize_program(program: &mut Program) -> Result<bool> {
    UnreachableBlockElim::new(program).run()
}

struct UnreachableBlockElim<'a> {
    program: &'a mut Program,
}

impl<'a> UnreachableBlockElim<'a> {
    fn new(program: &'a mut Program) -> Self {
        Self { program }
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
