use std::collections::{HashMap, HashSet};

use super::*;

// turn virtual backend module to phisic backend module
#[allow(unused)]
pub fn phisicalize(program: &mut Program) -> Result<(), BackendError> {
    for module in program.modules.iter_mut() {
        for func in module.funcs.iter_mut() {
            phisicalize_reg(func)?;

            // 为函数开头和结尾插入callee-save regs的保存和恢复
            handle_callee_save(func)?;

            // 为call指令前后插入caller-save regs的保存和恢复
            handle_caller_save(func)?;

            // entry和exit插入ra寄存器的保存和恢复
            handle_ra(func)?;

            // 为entry和exit插入栈的开辟和关闭,s0寄存器的保存和恢复
            handle_stack(func)?;

            // 替换所有使用的内存操作伪指令 为 实际的内存操作指令,比如load a0,[0-8] 修改为ld a0,0(sp)
            handle_mem(func)?;
        }
    }
    Ok(())
}
const fn tmp_i_regs() -> [Reg; 3] {
    [REG_T0, REG_T1, REG_T2]
}
const fn tmp_f_regs() -> [Reg; 3] {
    [REG_FT0, REG_FT1, REG_FT2]
}

fn phisicalize_reg(func: &mut Func) -> Result<()> {
    // count stack size: 统计栈大小,首先遍历每个块每条指令,统计中函数调用的最大栈大小
    let mut stack_allocator = func
        .stack_allocator_mut()
        .take()
        .expect("msg: stack allocator not found");
    // 该情况下仅仅使用临时寄存器参与运算
    let i_regs = tmp_i_regs();
    let f_regs = tmp_f_regs();
    // 对于遇到的每个寄存器,为其分配栈上空间
    let mut v_ss: HashMap<Reg, StackSlot> = HashMap::new();
    for v_r in func.v_regs() {
        v_ss.insert(v_r, stack_allocator.alloc(8));
    }
    func.stack_allocator_mut().replace(stack_allocator);

    // 对于每个块,遍历每个指令,涉及到栈的指令,将其替换为栈上的指令
    for bb in func.iter_bbs_mut() {
        let mut new_insts: Vec<Inst> = Vec::new();
        for inst in bb.insts() {
            // dbg!("process inst:", inst.gen_asm());
            let mut tmp_used: HashSet<Reg> = HashSet::new();
            let uses = inst.uses();
            let defs = inst.defs();
            let mut new_inst = inst.clone();
            // 首先加载uses中的寄存器需要的值
            for u in uses {
                if u.is_phisic() {
                    continue;
                }
                let ss = v_ss.get(u).unwrap();
                let replace = if u.is_usual() {
                    let i_r = i_regs.iter().find(|&&r| !tmp_used.contains(&r)).unwrap();
                    *i_r
                } else {
                    let f_r = f_regs.iter().find(|&&r| !tmp_used.contains(&r)).unwrap();
                    *f_r
                };
                tmp_used.insert(replace);
                new_inst.replace_use(*u, replace)?;
                // dbg!("replace use:", u.gen_asm(), replace.gen_asm());
                new_insts.push(LoadInst::new(replace, *ss).with_8byte().into());
            }

            // 处理使用临时寄存器替换虚拟寄存器之后要把值store回栈中
            let mut store_back: Option<Inst> = None;
            for d in defs {
                if d.is_phisic() {
                    continue;
                }
                let ss = v_ss.get(d).unwrap();
                let replace = if d.is_usual() {
                    let i_r = i_regs.iter().find(|&&r| !tmp_used.contains(&r)).unwrap();
                    *i_r
                } else {
                    let f_r = f_regs.iter().find(|&&r| !tmp_used.contains(&r)).unwrap();
                    *f_r
                };
                tmp_used.insert(replace);
                new_inst.replace_def(*d, replace)?;
                store_back = Some(StoreInst::new(*ss, replace).with_8byte().into());
            }
            new_insts.push(new_inst);

            if let Some(store_back) = store_back {
                new_insts.push(store_back);
            }
        }
        *bb.insts_mut() = new_insts;
    }
    Ok(())
}

#[allow(unused)]
fn handle_caller_save(func: &mut Func) -> Result<()> {
    // 统计代码中使用到的caller save寄存器,然后在函数调用前后保存和恢复这些寄存器
    let mut regs: HashSet<Reg> = HashSet::new();
    for bb in func.iter_bbs() {
        for inst in bb.insts() {
            let uses = inst.uses();
            let defs = inst.defs();
            regs.extend(uses.iter().filter(|r| r.is_phisic()).cloned());
            regs.extend(defs.iter().filter(|r| r.is_phisic()).cloned());
        }
    }
    regs.retain(|r| Reg::callee_save_regs().contains(r));

    // 为这些物理寄存器分配栈上空间,并在函数调用前后保存和恢复这些寄存器
    let mut stack_allocator = func
        .stack_allocator_mut()
        .take()
        .expect("msg: stack allocator not found");
    let mut reg_ss = HashMap::new();
    for r in regs.iter() {
        let ss = stack_allocator.alloc(8);
        reg_ss.insert(*r, ss);
    }
    func.stack_allocator_mut().replace(stack_allocator);

    // 为每个函数调用前后插入保存和恢复寄存器的指令
    for bb in func.iter_bbs_mut() {
        let mut new_insts = Vec::new();
        for inst in bb.insts() {
            match inst {
                Inst::Call(call) => {
                    for (r, ss) in reg_ss.iter() {
                        let sd = SdInst::new(*r, ss.start().into(), REG_SP);
                        new_insts.push(sd.into());
                    }
                    new_insts.push(inst.clone());
                    for (r, ss) in reg_ss.iter() {
                        let ld = LdInst::new(*r, ss.start().into(), REG_SP);
                        new_insts.push(ld.into());
                    }
                }
                _ => {
                    new_insts.push(inst.clone());
                }
            }
        }
        *bb.insts_mut() = new_insts;
    }

    Ok(())
}

#[allow(unused)]
fn handle_callee_save(func: &mut Func) -> Result<()> {
    // 统计代码中使用到的callee save寄存器,然后在函数开头和结尾保存和恢复这些寄存器
    let mut regs: HashSet<Reg> = HashSet::new();
    for bb in func.iter_bbs() {
        for inst in bb.insts() {
            let uses = inst.uses();
            let defs = inst.defs();
            regs.extend(uses.iter().filter(|r| r.is_phisic()).cloned());
            regs.extend(defs.iter().filter(|r| r.is_phisic()).cloned());
        }
    }
    regs.retain(|r| Reg::callee_save_regs().contains(r));

    // 为这些物理寄存器分配栈上空间
    let mut stack_allocator = func
        .stack_allocator_mut()
        .take()
        .expect("msg: stack allocator not found");
    let mut reg_ss = HashMap::new();
    for r in regs.iter() {
        let ss = stack_allocator.alloc(8);
        reg_ss.insert(*r, ss);
    }
    func.stack_allocator_mut().replace(stack_allocator);

    // 为函数开头和结尾插入保存和恢复寄存器的指令
    let entry = func.entry_mut().insts_mut();
    reg_ss
        .iter()
        .map(|(r, ss)| StoreInst::new(*ss, *r))
        .for_each(|i| entry.push(i.into()));

    let exit_bbs = func.exit_bbs_mut();
    let mut load_back = reg_ss.iter().map(|(r, ss)| LoadInst::new(*r, *ss));
    for bb in exit_bbs {
        load_back.clone().for_each(|i| {
            let n = bb.insts().len();
            bb.insts_mut().insert(n - 1, i.into());
        });
    }

    Ok(())
}

fn handle_ra(func: &mut Func) -> Result<()> {
    // if func is not a caller, then no need to handle ra
    if !func.is_caller() {
        return Ok(());
    }
    // insert store ra
    _ = func.stack_allocator_mut().iter_mut().map(|sa| sa.alloc(8));
    let sd_ra = SdInst::new(REG_RA, (-8).into(), REG_S0);
    func.entry_mut().insts_mut().insert(0, sd_ra.into());

    // insert load back ra
    func.exit_bbs_mut().for_each(|bb| {
        let ld_ra = LdInst::new(REG_RA, (-8).into(), REG_S0);
        let n = bb.insts().len();
        bb.insts_mut().insert(n - 1, ld_ra.into());
    });
    Ok(())
}

fn final_stack_size(func: &Func) -> Result<u32> {
    let r = func.stack_allocator().expect("").allocated();
    let r = (r as u64 + 15) & !15;
    if r.ge(&(u32::MAX as u64)) {
        return Err(anyhow!("stack size overflow"));
    }
    Ok(r as u32)
}

fn handle_stack(func: &mut Func) -> Result<()> {
    // alloc stack for s0, in fact, we could choose not to store-restore s0
    _ = func.stack_allocator_mut().iter_mut().map(|sa| sa.alloc(8));

    let stack_size = final_stack_size(func)? as i64;
    let record_s0 = SdInst::new(REG_S0, (-16).into(), REG_SP);
    let update_s0 = MvInst::new(REG_S0.into(), REG_SP.into());
    let open_stack = AddInst::new(REG_SP.into(), REG_SP.into(), (-stack_size).into());

    let entry = func.entry_mut().insts_mut();
    [record_s0.into(), update_s0.into(), open_stack.into()]
        .into_iter()
        .rev()
        .for_each(|i| {
            entry.insert(0, i);
        });

    let close_stack: Inst = AddInst::new(REG_SP.into(), REG_SP.into(), stack_size.into()).into();
    let restore_s0: Inst = LdInst::new(REG_S0, (-16).into(), REG_S0).into();
    func.exit_bbs_mut().for_each(|bb| {
        let n = bb.insts().len();
        bb.insts_mut().insert(n - 1, close_stack.clone());
        bb.insts_mut().insert(n, restore_s0.clone());
    });

    Ok(())
}

fn handle_mem(func: &mut Func) -> Result<()> {
    let stack_size = final_stack_size(func)?;
    for bb in func.iter_bbs_mut() {
        for inst in bb.insts_mut() {
            match inst {
                Inst::Load(load) => *inst = load.phisicalize(stack_size)?,
                Inst::Store(store) => *inst = store.phisicalize(stack_size)?,
                _ => {}
            };
        }
    }
    Ok(())
}
