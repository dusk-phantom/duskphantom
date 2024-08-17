use core::num;
use std::hash::Hash;

use checker::FuncChecker;
use graph::UdGraph;

mod graph_color;
pub use graph_color::*;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::fprintln;

use super::*;

pub fn handle_reg_alloc(func: &mut Func) -> Result<()> {
    checker::TightTerm.check_func(func);
    let mut reg_graph = Func::reg_interfere_graph(func)?;
    let dtd = func.def_then_def();
    let could_merge = collect_mergeable_regs(func, &reg_graph);
    if let Ok(colors) = try_perfect_alloc(&reg_graph, &dtd, &could_merge) {
        println!("### perfect alloc");
        apply_colors(func, colors);
    } else {
        let (colors, spills) = reg_alloc(&reg_graph, free_iregs(), free_fregs())?;
        apply_colors(func, colors);
        apply_spills(func, spills);
    }
    // 删除因为寄存器合并而产生的冗余指令

    Ok(())
}

// item 如（(r1, r2), 1) 表示r1和r2可以合并,且合并后能够减少的指令数为1
pub fn collect_mergeable_regs(
    func: &Func,
    graph: &FxHashMap<Reg, FxHashSet<Reg>>,
) -> Vec<((Reg, Reg), usize)> {
    // 如果两个虚拟寄存器合并为一个物理寄存器,能够减少指令,那么合并它们
    let mut could_merge: FxHashMap<(Reg, Reg), usize> = FxHashMap::default();
    let add_to_could_merge =
        |could_merge: &mut FxHashMap<(Reg, Reg), usize>, dst: &Operand, src: &Operand| {
            if let (Operand::Reg(dst), Operand::Reg(src)) = (dst, src) {
                if let Some(v) = could_merge.get_mut(&(*dst, *src)) {
                    *v += 1;
                } else if let Some(v) = could_merge.get_mut(&(*src, *dst)) {
                    *v += 1;
                } else {
                    could_merge.insert((*dst, *src), 1);
                }
            }
        };

    for bb in func.iter_bbs() {
        for inst in bb.insts() {
            match inst {
                Inst::Add(add) => {
                    if op_eq_zero(add.rhs()) {
                        add_to_could_merge(&mut could_merge, add.dst(), add.lhs());
                    }
                }
                Inst::Sub(sub) => {
                    if op_eq_zero(sub.rhs()) {
                        add_to_could_merge(&mut could_merge, sub.dst(), sub.lhs());
                    }
                }
                Inst::Mul(mul) => {
                    if op_eq_one(mul.rhs()) {
                        add_to_could_merge(&mut could_merge, mul.dst(), mul.lhs());
                    }
                }
                Inst::Div(div) => {
                    if op_eq_one(div.rhs()) {
                        add_to_could_merge(&mut could_merge, div.dst(), div.lhs());
                    }
                }
                Inst::Sll(sll) => {
                    if op_eq_zero(sll.rhs()) {
                        add_to_could_merge(&mut could_merge, sll.dst(), sll.lhs());
                    }
                }
                Inst::Srl(srl) => {
                    if op_eq_zero(srl.rhs()) {
                        add_to_could_merge(&mut could_merge, srl.dst(), srl.lhs());
                    }
                }
                Inst::SRA(sra) => {
                    if op_eq_zero(sra.rhs()) {
                        add_to_could_merge(&mut could_merge, sra.dst(), sra.lhs());
                    }
                }
                Inst::Mv(mv) => {
                    add_to_could_merge(&mut could_merge, mv.dst(), mv.src());
                }
                _ => {}
            }
        }
    }
    let mut could_merge: Vec<((Reg, Reg), usize)> = could_merge.into_iter().collect();
    could_merge.sort_by_key(|e| e.1);
    could_merge.reverse();
    could_merge
}

/// 给冲突图增加约束以实现寄存器合并
/// 如果两个虚拟寄存器或者一个虚拟寄存器和一个物理寄存器被认为应该分配到同一个物理寄存器上,那么就把它们在冲突图中的冲突列表合并并
/// 设置为他们各自的冲突列表
pub fn merge_regs(
    graph: &mut FxHashMap<Reg, FxHashSet<Reg>>,
    could_merge: &[((Reg, Reg), usize)],
    num_available_iregs: usize,
    num_available_fregs: usize,
) {
    for ((r1, r2), _) in could_merge.iter().rev() {
        // 如果两个寄存器都是物理寄存器,那么不需要合并,也不能合并
        if r1.is_physical() && r2.is_physical() {
            continue;
        }
        let mut r1_inter = graph.get(r1).unwrap().clone();
        let r2_inter = graph.get(r2).unwrap().clone();
        r1_inter.extend(&r2_inter);
        r1_inter.retain(|r| !special_regs().contains(r));
        let num_available = if r1.is_usual() {
            num_available_iregs
        } else {
            num_available_fregs
        };
        // 移除不需要考虑
        let num_inter = r1_inter
            .iter()
            .filter(|r| !special_regs().contains(r) && r.is_usual() == r1.is_usual())
            .count();
        // 如果合并两个寄存器(也就是给他们分配一样的颜色)之后,寄存器压力小于可分配物理寄存器的数量,那么认为不会降低可着色性
        // 则把新的冲突列表更新给r1,r2
        if num_inter < num_available {
            graph.insert(*r1, r1_inter.clone());
            graph.insert(*r2, r1_inter);
        }
    }
}

/// 删除因为寄存器合并而产生的冗余指令
pub fn remove_redundant_insts(func: &mut Func) {
    for bb in func.iter_bbs_mut() {
        bb.insts_mut().retain(|i| {
            println!("remove redundant inst: {:?}", i.gen_asm());
            !is_redundant_inst(i)
        });
    }
}

/// 判断一个指令是否是冗余的
pub fn is_redundant_inst(inst: &Inst) -> bool {
    match inst {
        Inst::Add(add) => op_eq_zero(add.rhs()) && add.dst() == add.lhs(),
        Inst::Sub(sub) => op_eq_zero(sub.rhs()) && sub.dst() == sub.lhs(),
        Inst::Mul(mul) => op_eq_one(mul.rhs()) && mul.dst() == mul.lhs(),
        Inst::Div(div) => op_eq_one(div.rhs()) && div.dst() == div.lhs(),
        Inst::Sll(sll) => op_eq_zero(sll.rhs()) && sll.dst() == sll.lhs(),
        Inst::Srl(srl) => op_eq_zero(srl.rhs()) && srl.dst() == srl.lhs(),
        Inst::SRA(sra) => op_eq_zero(sra.rhs()) && sra.dst() == sra.lhs(),
        Inst::Mv(mv) => mv.dst() == mv.src(),
        _ => false,
    }
}

fn op_eq_zero(op: &Operand) -> bool {
    if let Operand::Reg(r) = op {
        r == &REG_ZERO
    } else if let Operand::Imm(i) = op {
        i == &0.into()
    } else {
        false
    }
}

fn op_eq_one(op: &Operand) -> bool {
    if let Operand::Imm(i) = op {
        i == &1.into()
    } else {
        false
    }
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
