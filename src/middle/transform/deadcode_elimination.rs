use anyhow::Result;

use crate::middle::ir::instruction::InstType;
use crate::middle::ir::{BBPtr, FunPtr, InstPtr, Operand};
use crate::middle::{ir, Program};

#[allow(unused)]
pub fn optimize_program(program: &mut Program) -> Result<()> {
    deadcode_elimination(&mut program.module);
    Ok(())
}

#[allow(unused)]
pub fn deadcode_elimination(modu: &mut ir::Module) {
    modu.functions
        .iter()
        .filter(|f| !f.is_lib())
        .for_each(|x| deadcode_elimination_func(*x));
}

pub fn deadcode_elimination_func(func: FunPtr) {
    func.bfs_iter_rev().for_each(deadcode_elimination_block);
}
pub fn deadcode_elimination_block(bb: BBPtr) {
    bb.iter().for_each(deadcode_elimination_inst);
}
pub fn deadcode_elimination_inst(mut inst: InstPtr) {
    if !inst.get_user().is_empty() {
        return;
    }
    if has_side_effect(inst) {
        return;
    }
    let ops: Vec<Operand> = inst.get_operand().into();
    inst.remove_self();
    for ele in ops {
        match ele {
            Operand::Instruction(i) => deadcode_elimination_inst(i),
            Operand::Global(_) => {}
            _ => {}
        }
    }
}
fn has_side_effect(inst: InstPtr) -> bool {
    match inst.get_type() {
        // TODO pure function analysis
        InstType::Store | InstType::Call | InstType::Ret => true,
        _ => false,
    }
}
