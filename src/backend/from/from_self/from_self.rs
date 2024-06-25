use var::ArrVar;

use super::*;

use crate::errors::*;
use crate::middle;
use crate::middle::ir::Constant;

#[allow(unused)]
pub fn gen(program: &middle::Program) -> Result<prog::Program, BackendError> {
    let mut global_vars = Vec::new();
    let mut funcs = Vec::new();
    let llvm = &program.module;
    // dbg!(&llvm.types);
    // dbg!(&global_vars);

    for f in &llvm.functions {
        // dbg!(&f);
        let args: Vec<String> = f.params.iter().map(|p| p.name.to_string()).collect();
        let mut reg_gener = RegGenerator::new();
        let mut regs: HashMap<Name, Reg> = HashMap::new();
        let ret_ty = &f.return_type;
        let mut stack_allocator = StackAllocator::new();
        let mut stack_slots: HashMap<Name, StackSlot> = HashMap::new();
        let mut bb = f.entry;
        // let entry = build_bb(
        //     bb,
        //     &mut stack_allocator,
        //     &mut stack_slots,
        //     &mut reg_gener,
        //     &mut regs,
        // )?;
        // let mut m_f = Func::new(f.name.to_string(), args, entry);
        // // dbg!(&ret_ty);
        // for bb in &f.basic_blocks[1..] {
        //     let m_bb = build_bb(
        //         bb,
        //         &mut stack_allocator,
        //         &mut stack_slots,
        //         &mut reg_gener,
        //         &mut regs,
        //     )?;
        //     m_f.push_bb(m_bb);
        // }
        // // count stack size,
        // let stack_size = stack_allocator.allocated();
        // // align to 16
        // let stack_size = if stack_size % 16 == 0 {
        //     stack_size
        // } else {
        //     stack_size - (stack_size % 16) + 16
        // };
        // funcs.push(m_f);
    }

    // TODO
    Ok(prog::Program {
        entry: None,
        modules: vec![],
    })
}
