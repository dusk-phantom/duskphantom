use std::collections::HashMap;

use super::*;

// turn virtual backend module to phisic backend module
#[allow(unused)]
pub fn phisicalize(program: &mut Program) -> Result<(), BackendError> {
    for module in program.modules.iter_mut() {
        for func in module.funcs.iter_mut() {
            // count stack size: 统计栈大小,首先遍历每个块每条指令,统计中函数调用的最大栈大小
            let mut stack_allocator = func
                .stack_allocator_mut()
                .take()
                .expect("msg: stack allocator not found");
            // 该情况下仅仅使用临时寄存器参与运算
            let i_regs = [REG_T0, REG_T1, REG_T2];
            let f_regs = [REG_FT0, REG_FT1, REG_FT2];
            // 对于遇到的每个寄存器,为其分配栈上空间
            let mut v_ss: HashMap<Reg, StackSlot> = HashMap::new();
            for v_r in func.v_regs() {
                v_ss.insert(v_r, stack_allocator.alloc(8));
            }
            // 对于每个块,遍历每个指令,涉及到栈的指令,将其替换为栈上的指令
            for bb in func.iter_bbs_mut() {
                let mut new_insts: Vec<Inst> = Vec::new();
                for inst in bb.insts() {
                    todo!();
                }
                *bb.insts_mut() = new_insts;
            }

            // TODO: 为函数开头和结尾插入callee-save regs的保存和恢复

            // TODO: 为call指令前后插入caller-save regs的保存和恢复

            // TODO: 计算此时的栈空间,16字节对齐

            // TODO: 为函数开头和结尾插入栈的开辟和关闭,ra寄存器的保存和恢复,s0寄存器的保存和恢复

            // TODO: 替换所有使用的内存操作伪指令 为 实际的内存操作指令,比如load a0,[0-8] 修改为ld a0,0(sp)
        }
    }
    Ok(())
}
