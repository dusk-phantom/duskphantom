use crate::middle::ir;
use crate::middle::ir::{BBPtr, FunPtr, InstPtr, Operand};

pub fn deadcode_elimination(modu: &mut ir::Module) {
    modu.functions
        .iter()
        .for_each(|x| deadcode_elimination_func(x.clone()));
}

pub fn deadcode_elimination_func(func: FunPtr) {
    func.bfs_iter_rev()
        .for_each(|x| deadcode_elimination_block(x.clone()));
}
pub fn deadcode_elimination_block(bb: BBPtr) {
    bb.iter().for_each(|x| deadcode_elimination_inst(x.clone()));
}
pub fn deadcode_elimination_inst(mut inst: InstPtr) {
    if inst.get_user().len() != 0 {
        return;
    }
    let ops: Vec<Operand> = inst.get_operand().into();
    inst.remove_self();
    for ele in ops {
        match ele {
            // Need to call deadcode_elimination again?
            Operand::Instruction(i) => deadcode_elimination_inst(i.clone()),
            // TODO: Other Operand
            _ => {}
        }
    }
}
