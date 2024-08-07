use anyhow::Result;

use crate::middle::{
    analysis::reachability::Reachability,
    ir::{instruction::InstType, Constant, FunPtr, Operand},
    Program,
};

#[allow(unused)]
pub fn optimize_program(program: &mut Program) -> Result<()> {
    UnreachableBlockElim::new(program).run();
    Ok(())
}

struct UnreachableBlockElim<'a> {
    program: &'a mut Program,
}

impl<'a> UnreachableBlockElim<'a> {
    fn new(program: &'a mut Program) -> Self {
        Self { program }
    }

    fn run(&mut self) {
        for func in self
            .program
            .module
            .functions
            .clone()
            .iter()
            .filter(|f| !f.is_lib())
        {
            self.process_function(*func);
        }
    }

    fn process_function(&mut self, func: FunPtr) {
        // Simplify "br" with constant condition to unconditional
        let bbs: Vec<_> = func.dfs_iter().collect();
        for bb in bbs.iter() {
            for mut inst in bb.iter() {
                if inst.get_type() == InstType::Br {
                    let cond = inst.get_operand().first().cloned();
                    if let Some(Operand::Constant(Constant::Bool(cond))) = cond {
                        // Rewire basic block
                        let mut parent_bb = inst.get_parent_bb().unwrap();
                        if cond {
                            parent_bb.remove_false_bb();
                        } else {
                            parent_bb.remove_true_bb();
                        }

                        // Replace instruction with unconditional jump
                        let new_inst = self.program.mem_pool.get_br(None);
                        inst.insert_after(new_inst);
                        inst.remove_self();
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
                }
            }
        }
    }
}
