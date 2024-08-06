use anyhow::Result;

use crate::middle::ir::instruction::InstType;
use crate::middle::ir::{BBPtr, FunPtr, InstPtr, Operand};
use crate::middle::Program;

#[allow(unused)]
pub fn optimize_program(program: &mut Program) -> Result<()> {
    DeadCodeElimination::new(program).deadcode_elimination();
    Ok(())
}

struct DeadCodeElimination<'a> {
    program: &'a mut Program,
}

impl<'a> DeadCodeElimination<'a> {
    fn new(program: &'a mut Program) -> Self {
        Self { program }
    }

    fn deadcode_elimination(&mut self) {
        self.program
            .module
            .functions
            .clone()
            .iter()
            .filter(|f| !f.is_lib())
            .for_each(deadcode_elimination_func);

        // Global variable does not require revisit, remove unused variables at the end
        self.program
            .module
            .global_variables
            .retain(|x| !x.get_user().is_empty());
    }
}

fn deadcode_elimination_func(func: &FunPtr) {
    // Use post order traversal to reduce revisits
    func.po_iter().for_each(deadcode_elimination_block);
}

fn deadcode_elimination_block(bb: BBPtr) {
    // Iterate forward so that next instruction is always valid
    bb.iter().for_each(deadcode_elimination_inst);
}

fn deadcode_elimination_inst(mut inst: InstPtr) {
    if !inst.get_user().is_empty() || has_side_effect(inst) {
        return;
    }
    let operands: Vec<_> = inst.get_operand().into();
    inst.remove_self();
    for op in operands {
        if let Operand::Instruction(inst) = op {
            deadcode_elimination_inst(inst);
        }
    }
}

fn has_side_effect(inst: InstPtr) -> bool {
    // TODO pure function analysis
    matches!(
        inst.get_type(),
        InstType::Store | InstType::Call | InstType::Ret | InstType::Br
    )
}
