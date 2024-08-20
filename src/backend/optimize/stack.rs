use rustc_hash::FxHashMap;

use super::*;

/// 栈空间操作: 处理ra,sp,s0,的保存和恢复, 以及使用t3寄存器处理访存指令off溢出
pub fn handle_stack(func: &mut Func) -> Result<()> {
    handle_stack_reorder(func)?;
    phisicalize::handle_ra(func)?;
    phisicalize::handle_stack(func)?;
    phisicalize::handle_mem(func)?;
    phisicalize::handle_offset_overflows(func)?;
    Ok(())
}

pub fn handle_stack_reorder(func: &mut Func) -> Result<()> {
    let mut old_ss: FxHashMap<StackSlot, usize> = FxHashMap::default();
    for bb in func.iter_bbs() {
        for inst in bb.insts() {
            if let Some(stack_slot) = inst.stack_slot().cloned() {
                *old_ss.entry(stack_slot).or_default() += 1;
            }
        }
    }
    let mut ss = old_ss.into_iter().collect::<Vec<_>>();
    ss.sort_by_key(|(_, count)| *count);

    let mut new_ssa = StackAllocator::new();
    let mut old_news = FxHashMap::default();
    new_ssa.alloc(func.max_callee_regs_stack);

    let mut pre_ss = Vec::new();
    let mut suf_ss = Vec::new();
    for (idx, (ss, _)) in ss.into_iter().rev().enumerate() {
        if idx % 2 == 0 {
            pre_ss.push(ss);
        } else {
            suf_ss.push(ss);
        }
    }
    for ss in pre_ss.into_iter().chain(suf_ss.into_iter().rev()) {
        let new_ss = new_ssa.alloc(ss.size());
        old_news.insert(ss, new_ss);
    }
    for bb in func.iter_bbs_mut() {
        for inst in bb.insts_mut() {
            if let Some(old_ss) = inst.stack_slot_mut() {
                let new_ss = old_news[old_ss];
                *old_ss = new_ss;
            }
        }
    }

    func.stack_allocator_mut().replace(new_ssa);

    Ok(())
}
