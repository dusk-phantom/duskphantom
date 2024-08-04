use super::*;
/// 处理调用者保存和被调用者保存
pub fn handle_caller_callee(func: &mut Func) -> Result<()> {
    handle_caller_save(func)?;
    handle_callee_save(func)?;
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
            regs.extend(uses.iter().filter(|r| r.is_physical()).cloned());
            regs.extend(defs.iter().filter(|r| r.is_physical()).cloned());
        }
    }
    regs.retain(|r| Reg::caller_save_regs().contains(r));
    regs.retain(|r| !tmp_i_regs().contains(r) && !tmp_f_regs().contains(r));

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
                    // 计算要在函数调用前后保护(保存和恢复)的寄存器
                    let mut to_protect = reg_ss.clone();
                    let mut call_defs = call.defs();
                    to_protect.retain(|r, _| !call_defs.contains(&r));

                    // 为这些寄存器在call指令前后插入保存和恢复指令
                    for (r, ss) in to_protect.iter() {
                        let sd = StoreInst::new(*ss, *r).with_8byte();
                        new_insts.push(sd.into());
                    }
                    new_insts.push(inst.clone());
                    for (r, ss) in to_protect.iter() {
                        let ld = LoadInst::new(*r, *ss).with_8byte();
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
            regs.extend(uses.iter().filter(|r| r.is_physical()).cloned());
            regs.extend(defs.iter().filter(|r| r.is_physical()).cloned());
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
        .map(|(r, ss)| StoreInst::new(*ss, *r).with_8byte())
        .for_each(|i| entry.insert(0, i.into()));

    let exit_bbs = func.exit_bbs_mut();

    let mut load_back = reg_ss
        .iter()
        .map(|(r, ss)| LoadInst::new(*r, *ss).with_8byte());
    for bb in exit_bbs {
        load_back.clone().for_each(|i| {
            bb.insert_before_term(i.into());
        });
    }

    Ok(())
}
