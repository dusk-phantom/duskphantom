use crate::{config::CONFIG, fprintln};

use super::irs::*;
use std::collections::{HashMap, HashSet, VecDeque};

pub mod analysis;
#[allow(unused)]
pub mod inst_combine;

pub mod pre_inst_split;

pub mod post_inst_split;

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
            if CONFIG.num_parallel_for_func_gen_asm <= 1 {
                println!("num_parallel_for_func_gen_asm <= 1,run in single thread");
                m.funcs.iter_mut().try_for_each(optimize_func)?;
            } else {
                let thread_pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(CONFIG.num_parallel_for_func_gen_asm)
                    .build()
                    .unwrap();
                thread_pool.install(|| m.funcs.par_iter_mut().try_for_each(optimize_func))?;
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
fn test_symplify_and_desimplify_term(func: &mut Func) {
    fprintln!("2.s", "{}", func.gen_asm());
    for i in 0..1000 {
        func.simplify_term();
        fprintln!("2.s", "{}", func.gen_asm());
        func.desimplify_term();
        fprintln!("2.s", "{}", func.gen_asm());
    }
}

#[allow(unused)]
pub fn optimize_func(func: &mut Func) -> Result<()> {
    // inst combine? 匹配一些模式,将多条指令合并成一条
    fprintln!("log/before_inst_combine.s", "{}", func.gen_asm());
    inst_combine::handle_inst_combine(func)?;

    // inst split? 将一条指令拆分成多条
    pre_inst_split::handle_mul_div_opt(func)?;

    phisicalize::handle_illegal_inst(func)?;

    phisicalize::handle_long_jump(func, &REG_T0, 20_0000);

    pre_inst_split::handle_pre_split_li(func)?;

    fprintln!("log/after_inst_scheduling.s", "{}", func.gen_asm());
    // register allocation
    reg_alloc::handle_reg_alloc(func)?;
    fprintln!("log/after_reg_alloc.s", "{}", func.gen_asm());

    // processing caller-save and callee-save
    caller_callee::handle_caller_callee(func)?;

    // processing stack frame's opening and closing
    stack::handle_stack(func)?;

    fprintln!("log/before_split_li.s", "{}", func.gen_asm());
    post_inst_split::post_handle_inst_split(func)?;
    fprintln!("log/after_split_li.s", "{}", func.gen_asm());

    // inst scheduling
    schedule::handle_inst_scheduling(func)?;

    func.simplify_term();
    Ok(())
}
