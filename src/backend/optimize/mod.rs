use super::irs::*;
use std::collections::{HashMap, HashSet, VecDeque};

#[allow(unused)]
pub fn optimize(program: &mut prog::Program) -> Result<()> {
    #[cfg(feature = "backend_opt")]
    {
        for m in self.program.clone().as_mut().modules.iter_mut() {
            for f in m.funcs.iter_mut() {
                optimize_func(f)?;
            }
        }
    }
    #[cfg(not(feature = "backend_opt"))]
    {
        phisicalize::phisicalize(program); // 直接物理化
    }
    Ok(())
}

#[allow(unused)]
pub fn optimize_func(func: &mut Func) -> Result<()> {
    // inst split? 将一条指令拆分成多条

    // inst combine? 匹配一些模式,将多条指令合并成一条
    inst_combine::handle_inst_combine(func)?;

    // inst scheduling
    schedule::handle_inst_scheduling(func)?;
    // register allocation
    reg_alloc::handle_reg_alloc(func)?;
    // processing caller-save and callee-save
    caller_callee::handle_caller_callee(func)?;

    // block reordering
    block_reorder::handle_block_reordering(func)?;

    // processing stack frame's opening and closing
    stack::handle_stack_frame(func)?;
    Ok(())
}

#[allow(unused)]
pub mod inst_combine;

#[allow(unused)]
pub mod inst_split;

#[allow(unused)]
pub mod schedule;

#[allow(unused)]
pub mod reg_alloc;

#[allow(unused)]
pub mod caller_callee;

#[allow(unused)]
pub mod block_reorder;

#[allow(unused)]
pub mod stack;
