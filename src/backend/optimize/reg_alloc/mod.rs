use core::num;
use std::hash::Hash;

use graph::UdGraph;

mod graph_color;
pub use graph_color::*;

use crate::fprintln;

use super::*;

pub fn handle_reg_alloc(func: &mut Func) -> Result<()> {
    // if func.line() > 10000 {
    let mut reg_graph = Func::reg_interfere_graph(func)?;
    let dtd = func.def_then_def();
    if let Ok(colors) = try_perfect_alloc(&reg_graph, &dtd) {
        apply_colors(func, colors);
        return Ok(());
    }
    let (colors, spills) = reg_alloc(&reg_graph, free_iregs(), free_fregs())?;
    apply_colors(func, colors);
    apply_spills(func, spills);
    // } else {
    //     let mut reg_graph = Func::reg_interfere_graph2(func)?;
    //     let dtd = func.def_then_def2();
    //     if let Ok(colors) = try_perfect_alloc2(&reg_graph, &dtd) {
    //         apply_colors(func, colors);
    //         return Ok(());
    //     }
    //     let (colors, spills) = reg_alloc2(&reg_graph, free_iregs(), free_fregs())?;
    //     apply_colors(func, colors);
    //     apply_spills(func, spills);
    // }
    Ok(())
}

/// 能够用于寄存器分配的寄存器,也就是除了特殊寄存器以外的寄存器, 这里的特殊寄存器包括: zero, ra, sp, gp, tp,s0,t0-t3 <br>
/// 其中t0-t3是临时寄存器,t0-t2用于处理spill的虚拟寄存器,t3用于计算内存操作指令off溢出时的地址 <br>
/// s0是栈帧指针,用于保存调用者保存的寄存器 <br>
/// ...
pub fn free_iregs() -> &'static [Reg; 22] {
    &[
        // usual registers
        REG_S1, REG_A0, REG_A1, REG_A2, REG_A3, REG_A4, REG_A5, REG_A6, REG_A7, REG_S2, REG_S3,
        REG_S4, REG_S5, REG_S6, REG_S7, REG_S8, REG_S9, REG_S10, REG_S11, REG_T4, REG_T5, REG_T6,
    ]
}

/// 除了ft0-ft2用于处理spill的虚拟寄存器,其他的都可以自由用于寄存器分配
pub fn free_fregs() -> &'static [Reg; 29] {
    // usual registers
    &[
        // float registers
        REG_FT3, REG_FT4, REG_FT5, REG_FT6, REG_FT7, REG_FS0, REG_FS1, REG_FA0, REG_FA1, REG_FA2,
        REG_FA3, REG_FA4, REG_FA5, REG_FA6, REG_FA7, REG_FS2, REG_FS3, REG_FS4, REG_FS5, REG_FS6,
        REG_FS7, REG_FS8, REG_FS9, REG_FS10, REG_FS11, REG_FT8, REG_FT9, REG_FT10, REG_FT11,
    ]
}

/// 自由通用寄存器 加上 临时通用寄存器
pub fn free_iregs_with_tmp() -> &'static [Reg; 25] {
    &[
        /* tmp usual regs: */ REG_T0, REG_T1, REG_T2, /* free usual regs: */ REG_S1,
        REG_A0, REG_A1, REG_A2, REG_A3, REG_A4, REG_A5, REG_A6, REG_A7, REG_S2, REG_S3, REG_S4,
        REG_S5, REG_S6, REG_S7, REG_S8, REG_S9, REG_S10, REG_S11, REG_T4, REG_T5, REG_T6,
    ]
}

/// 自由浮点寄存器 加上 临时浮点寄存器
pub fn free_fregs_with_tmp() -> &'static [Reg; 32] {
    &[
        /* tmp float regs: */ REG_FT0, REG_FT1, REG_FT2, /* free float regs: */ REG_FT3,
        REG_FT4, REG_FT5, REG_FT6, REG_FT7, REG_FS0, REG_FS1, REG_FA0, REG_FA1, REG_FA2, REG_FA3,
        REG_FA4, REG_FA5, REG_FA6, REG_FA7, REG_FS2, REG_FS3, REG_FS4, REG_FS5, REG_FS6, REG_FS7,
        REG_FS8, REG_FS9, REG_FS10, REG_FS11, REG_FT8, REG_FT9, REG_FT10, REG_FT11,
    ]
}

/// 特殊作用的寄存器
pub fn special_regs() -> &'static [Reg; 7] {
    &[
        REG_ZERO, // zero register
        REG_RA,   // return address
        REG_SP,   // stack pointer
        REG_GP,   // global pointer
        REG_TP,   // thread pointer
        REG_S0,   // stack frame pointer
        REG_T3,   // temp register for address overflow
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::backend::irs::Reg;

    use super::{free_fregs, free_fregs_with_tmp, free_iregs, free_iregs_with_tmp};

    #[test]
    fn no_duplicate() {
        let check = |regs: &[Reg]| {
            let r_set: HashSet<Reg> = regs.iter().cloned().collect();
            assert!(r_set.len() == regs.len());
        };
        check(free_fregs());
        check(free_iregs());
        check(free_fregs_with_tmp());
        check(free_iregs_with_tmp());
    }
}
