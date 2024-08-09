use crate::fprintln;

use super::irs::*;
use std::collections::{HashMap, HashSet, VecDeque};

pub mod analysis;
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

#[allow(unused)]
pub fn optimize(program: &mut prog::Program) -> Result<()> {
    #[cfg(feature = "backend_opt")]
    {
        for m in program.modules.iter_mut() {
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
    // inst combine? 匹配一些模式,将多条指令合并成一条
    fprintln!("log/before_inst_combine.s", "{}", func.gen_asm());
    inst_combine::handle_inst_combine(func)?;

    // inst split? 将一条指令拆分成多条
    inst_split::handle_mul_div_opt(func)?;

    phisicalize::handle_illegal_inst(func)?;

    // inst scheduling
    // schedule::handle_inst_scheduling(func)?;

    phisicalize::handle_long_jump(func, &REG_T0, 20_0000);

    fprintln!("log/after_inst_scheduling.s", "{}", func.gen_asm());
    // register allocation
    reg_alloc::handle_reg_alloc(func)?;
    fprintln!("log/after_reg_alloc.s", "{}", func.gen_asm());

    // processing caller-save and callee-save
    caller_callee::handle_caller_callee(func)?;

    // processing stack frame's opening and closing
    stack::handle_stack(func)?;

    block_reorder::handle_single_jmp(func)?;
    Ok(())
}
