use std::collections::{HashMap, HashSet};

use crate::fprintln;

use super::irs::*;

// turn virtual backend module to phisic backend module
#[allow(unused)]
pub fn phisicalize(program: &mut Program) -> Result<(), BackendError> {
    // return Ok(()); // debug
    for module in program.modules.iter_mut() {
        for func in module.funcs.iter_mut() {
            phisicalize_func(func)?;
        }
    }
    Ok(())
}

pub fn phisicalize_func(func: &mut Func) -> Result<()> {
    fprintln!("log/before_handle_unlegal_inst.log", "{}", func.gen_asm());
    handle_illegal_inst(func)?;

    fprintln!("log/before_phisicalize.log", "{}", func.gen_asm());

    phisicalize_reg(func)?;

    fprintln!("log/phisicalize_func.s", "{}", func.gen_asm());

    // 为函数开头和结尾插入callee-save regs的保存和恢复
    // handle_callee_save(func)?;
    // fprintln!("log/handle_callee_save.s", "{}", func.gen_asm());

    // // 为call指令前后插入caller-save regs的保存和恢复
    // handle_caller_save(func)?;
    // fprintln!("log/handle_caller_save.s", "{}", func.gen_asm());

    // entry和exit插入ra寄存器的保存和恢复
    handle_ra(func)?;
    fprintln!("log/handle_ra.s", "{}", func.gen_asm());

    // 为entry和exit插入栈的开辟和关闭(通过sp的减少和增加实现),s0寄存器的保存和恢复
    handle_stack(func)?;
    fprintln!("log/handle_stack.s", "{}", func.gen_asm());

    // 替换所有使用的内存操作伪指令 为 实际的内存操作指令,比如load a0,[0-8] 修改为ld a0,0(sp)
    handle_mem(func)?;
    fprintln!("log/handle_mem.s", "{}", func.gen_asm());

    // 处理load和store类型指令 使用的 地址偏移 超出范围的情况
    handle_offset_overflows(func)?;
    fprintln!("log/handle_offset_overflows.s", "{}", func.gen_asm());

    Ok(())
}

pub const fn tmp_i_regs() -> [Reg; 3] {
    [REG_T0, REG_T1, REG_T2]
}

pub const fn tmp_f_regs() -> [Reg; 3] {
    [REG_FT0, REG_FT1, REG_FT2]
}

/// process some inst to compliant with physical backend,
/// e.g. mul's rhs must be a register
/// e.g. sltu's rhs must be a register
/// e.g. div's rhs must be a register
pub fn handle_illegal_inst(func: &mut Func) -> Result<()> {
    let mut r_g = func
        .reg_gener_mut()
        .take()
        .ok_or(anyhow!("msg: reg_gener not found"))
        .with_context(|| context!())?;
    macro_rules! process_rhs_imm {
        ($inst:ident,$r_g:ident,$new_insts:ident) => {
            if let Operand::Imm(imm) = $inst.rhs() {
                let mid = $r_g.gen_virtual_usual_reg();
                let li = LiInst::new(mid.into(), imm.into());
                *$inst.rhs_mut() = Operand::Reg(mid);
                $new_insts.push(li.into());
                $new_insts.push($inst.clone().into());
            } else {
                $new_insts.push($inst.clone().into());
            }
        };
    }
    for bb in func.iter_bbs_mut() {
        let mut new_insts: Vec<Inst> = Vec::new();
        for inst in bb.insts_mut() {
            match inst {
                Inst::Sltu(sltu) => process_rhs_imm!(sltu, r_g, new_insts),
                Inst::Sgtu(sgtu) => process_rhs_imm!(sgtu, r_g, new_insts),
                Inst::Mul(mul) => process_rhs_imm!(mul, r_g, new_insts),
                Inst::Div(div) => process_rhs_imm!(div, r_g, new_insts),
                Inst::Rem(rem) => process_rhs_imm!(rem, r_g, new_insts),
                Inst::Sub(sub) => process_rhs_imm!(sub, r_g, new_insts),
                // TODO, divu
                _ => {
                    new_insts.push(inst.clone());
                }
            }
        }
        *bb.insts_mut() = new_insts;
    }

    func.reg_gener_mut().replace(r_g);
    Ok(())
}

pub fn phisicalize_reg(func: &mut Func) -> Result<()> {
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
                if u.is_physical() {
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
                if d.is_physical() {
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

pub fn handle_ra(func: &mut Func) -> Result<()> {
    // if func is not a caller, then no need to handle ra
    if !func.is_caller() {
        return Ok(());
    }

    // insert store ra
    func.stack_allocator_mut().iter_mut().for_each(|sa| {
        sa.alloc(8);
    });

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

pub fn handle_stack(func: &mut Func) -> Result<()> {
    // alloc stack for s0, in fact, we could choose not to store-restore s0
    func.stack_allocator_mut().iter_mut().for_each(|sa| {
        sa.alloc(8);
    });

    let mut to_insert_front: Vec<Inst> = vec![];

    let offset = if func.is_caller() { -16 } else { -8 };

    let stack_size = final_stack_size(func)? as i64;
    let record_s0 = SdInst::new(REG_S0, offset.into(), REG_SP);
    to_insert_front.push(record_s0.into());
    let update_s0 = MvInst::new(REG_S0.into(), REG_SP.into());
    to_insert_front.push(update_s0.into());

    let to_minus: Imm = (-stack_size).into();
    let to_minus = if to_minus.in_limit(12) {
        to_minus.into()
    } else {
        let li = LiInst::new(REG_T0.into(), to_minus.into());
        to_insert_front.push(li.into());
        REG_T0.into()
    };
    let open_stack = AddInst::new(REG_SP.into(), REG_SP.into(), to_minus);
    to_insert_front.push(open_stack.into());

    let entry = func.entry_mut().insts_mut();
    to_insert_front.into_iter().rev().for_each(|i| {
        entry.insert(0, i);
    });

    let mut insert_before_ret: Vec<Inst> = vec![];

    let close_stack: Inst = MvInst::new(REG_SP.into(), REG_S0.into()).into();
    insert_before_ret.push(close_stack);

    let offset: Imm = offset.into();
    if offset.in_limit(12) {
        let restore_s0: Inst = LdInst::new(REG_S0, offset, REG_S0).into();
        insert_before_ret.push(restore_s0);
    } else {
        let li = LiInst::new(REG_T0.into(), offset.into());
        insert_before_ret.push(li.into());
        let add = AddInst::new(REG_T1.into(), REG_S0.into(), REG_T0.into());
        insert_before_ret.push(add.into());
        let ld = LdInst::new(REG_S0, 0.into(), REG_T1);
        insert_before_ret.push(ld.into());
    }

    for exit_bb in func.exit_bbs_mut() {
        for i in insert_before_ret.iter() {
            exit_bb.insert_before_term(i.clone())?;
        }
    }

    Ok(())
}

pub fn handle_mem(func: &mut Func) -> Result<()> {
    let stack_size = final_stack_size(func)?;
    for bb in func.iter_bbs_mut() {
        for inst in bb.insts_mut() {
            match inst {
                Inst::Load(load) => *inst = load.phisicalize(stack_size)?,
                Inst::Store(store) => *inst = store.phisicalize(stack_size)?,
                Inst::LocalAddr(local_addr) => *inst = local_addr.phisicalize(stack_size)?,
                _ => {}
            };
        }
    }
    Ok(())
}

pub fn handle_offset_overflows(func: &mut Func) -> Result<()> {
    macro_rules! handle_offset_overflow {
        ($inst:ident,$inst_ty:ident,$new_insts:ident) => {
            if !$inst.offset().in_limit(12) {
                let li = LiInst::new(REG_T3.into(), $inst.offset().into());
                let add = AddInst::new(REG_T3.into(), REG_T3.into(), $inst.base().into());
                let new_ld = $inst_ty::new(*$inst.dst(), 0.into(), REG_T3);
                $new_insts.push(li.into());
                $new_insts.push(add.into());
                $new_insts.push(new_ld.into());
            } else {
                $new_insts.push($inst.clone().into());
            }
        };
    }

    for bb in func.iter_bbs_mut() {
        let mut new_insts: Vec<Inst> = Vec::new();
        for inst in bb.insts() {
            match inst {
                Inst::Ld(inst) => handle_offset_overflow!(inst, LdInst, new_insts),
                Inst::Sd(sd) => handle_offset_overflow!(sd, SdInst, new_insts),
                Inst::Lw(lw) => handle_offset_overflow!(lw, LwInst, new_insts),
                Inst::Sw(sw) => handle_offset_overflow!(sw, SwInst, new_insts),
                Inst::Add(add) => {
                    let rhs = add.rhs();
                    if let Operand::Imm(imm) = rhs {
                        if imm.in_limit(12) {
                            new_insts.push(inst.clone());
                        } else {
                            let li = LiInst::new(REG_T3.into(), imm.into());
                            let new_add =
                                AddInst::new(add.dst().clone(), add.lhs().clone(), REG_T3.into());
                            new_insts.push(li.into());
                            new_insts.push(new_add.into());
                        }
                    } else {
                        new_insts.push(inst.clone());
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
