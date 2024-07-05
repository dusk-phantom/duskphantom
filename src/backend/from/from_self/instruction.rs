use crate::utils::mem::ObjPtr;
use crate::{backend::*, ssa2tac_binary_float, ssa2tac_binary_usual};
use crate::{context, middle};

use crate::middle::ir::instruction::binary_inst::BinaryInst;
use crate::middle::ir::instruction::downcast_ref;
use crate::middle::ir::Instruction;

use super::*;

use anyhow::{Context, Result};

use anyhow::Ok;
use builder::IRBuilder;
use std::collections::HashMap;

impl IRBuilder {
    pub fn build_instruction(
        inst: &ObjPtr<Box<dyn middle::ir::Instruction>>,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Address, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
    ) -> Result<Vec<Inst>> {
        match inst.get_type() {
            middle::ir::instruction::InstType::Head => {
                Err(anyhow!("head should not be in backend")).with_context(|| context!())
            } // 应该是不能有 Head 出现的
            middle::ir::instruction::InstType::Alloca => {
                let alloca = downcast_ref::<middle::ir::instruction::memory_op_inst::Alloca>(
                    inst.as_ref().as_ref(),
                );
                Self::build_alloca_inst(alloca, stack_allocator, stack_slots)
            }
            middle::ir::instruction::InstType::Store => {
                let store = downcast_ref::<middle::ir::instruction::memory_op_inst::Store>(
                    inst.as_ref().as_ref(),
                );
                Self::build_store_inst(store, stack_slots, reg_gener, regs)
            }
            middle::ir::instruction::InstType::Add => {
                ssa2tac_binary_usual!(inst, regs, reg_gener, Add, Add, AddInst)
            }
            middle::ir::instruction::InstType::FAdd => {
                ssa2tac_binary_float!(inst, regs, reg_gener, FAdd, Add, AddInst)
            }
            middle::ir::instruction::InstType::Sub => {
                ssa2tac_binary_usual!(inst, regs, reg_gener, Sub, Sub, SubInst)
            }
            // 通过类型转换，可以做到: FAdd 的输入一定是 Float 类型的寄存器
            middle::ir::instruction::InstType::FSub => {
                ssa2tac_binary_float!(inst, regs, reg_gener, FSub, Sub, SubInst)
            }
            middle::ir::instruction::InstType::Mul => {
                ssa2tac_binary_usual!(inst, regs, reg_gener, Mul, Mul, MulInst)
            }
            middle::ir::instruction::InstType::FMul => {
                ssa2tac_binary_float!(inst, regs, reg_gener, FMul, Mul, MulInst)
            }
            middle::ir::instruction::InstType::SDiv => {
                ssa2tac_binary_usual!(inst, regs, reg_gener, SDiv, Div, DivInst)
            }
            middle::ir::instruction::InstType::SRem => {
                ssa2tac_binary_usual!(inst, regs, reg_gener, SRem, Rem, RemInst)
            }

            // TODO
            middle::ir::instruction::InstType::UDiv => todo!(),
            middle::ir::instruction::InstType::URem => todo!(),
            middle::ir::instruction::InstType::FDiv => todo!(),
            middle::ir::instruction::InstType::Shl => {
                ssa2tac_binary_usual!(inst, regs, reg_gener, Shl, Sll, SllInst)
            }
            middle::ir::instruction::InstType::LShr => {
                ssa2tac_binary_usual!(inst, regs, reg_gener, LShr, Srl, SrlInst)
            }
            middle::ir::instruction::InstType::AShr => {
                ssa2tac_binary_usual!(inst, regs, reg_gener, AShr, SRA, SraInst)
            }
            middle::ir::instruction::InstType::And => {
                ssa2tac_binary_usual!(inst, regs, reg_gener, And, And, AndInst)
            }
            middle::ir::instruction::InstType::Or => {
                ssa2tac_binary_usual!(inst, regs, reg_gener, Or, Or, OrInst)
            }
            middle::ir::instruction::InstType::Xor => {
                ssa2tac_binary_usual!(inst, regs, reg_gener, Xor, Xor, XorInst)
            }
            middle::ir::instruction::InstType::Ret => {
                let ret = downcast_ref::<middle::ir::instruction::terminator_inst::Ret>(
                    inst.as_ref().as_ref(),
                );
                Self::build_ret_inst(ret, regs)
            }
            middle::ir::instruction::InstType::Br => {
                let br = downcast_ref::<middle::ir::instruction::terminator_inst::Br>(
                    inst.as_ref().as_ref(),
                );
                Self::build_br_inst(br, regs, reg_gener)
            }
            middle::ir::instruction::InstType::Load => {
                let load = downcast_ref::<middle::ir::instruction::memory_op_inst::Load>(
                    inst.as_ref().as_ref(),
                );
                Self::build_load_inst(load, stack_slots, reg_gener, regs)
            }
            middle::ir::instruction::InstType::GetElementPtr => todo!(),
            middle::ir::instruction::InstType::ZextTo => todo!(),
            middle::ir::instruction::InstType::SextTo => todo!(),
            middle::ir::instruction::InstType::ItoFp => todo!(),
            middle::ir::instruction::InstType::FpToI => todo!(),
            middle::ir::instruction::InstType::ICmp => {
                let icmp = downcast_ref::<middle::ir::instruction::misc_inst::ICmp>(
                    inst.as_ref().as_ref(),
                );
                match icmp.op {
                    middle::ir::instruction::misc_inst::ICmpOp::Eq => todo!(),
                    middle::ir::instruction::misc_inst::ICmpOp::Ne => todo!(),
                    middle::ir::instruction::misc_inst::ICmpOp::Slt => todo!(),
                    middle::ir::instruction::misc_inst::ICmpOp::Sle => todo!(),
                    middle::ir::instruction::misc_inst::ICmpOp::Sgt => todo!(),
                    middle::ir::instruction::misc_inst::ICmpOp::Sge => todo!(),
                    middle::ir::instruction::misc_inst::ICmpOp::Ult => todo!(),
                    middle::ir::instruction::misc_inst::ICmpOp::Ule => todo!(),
                    middle::ir::instruction::misc_inst::ICmpOp::Ugt => todo!(),
                    middle::ir::instruction::misc_inst::ICmpOp::Uge => todo!(),
                }
                todo!()
            }
            middle::ir::instruction::InstType::FCmp => todo!(),
            middle::ir::instruction::InstType::Phi => todo!(),
            middle::ir::instruction::InstType::Call => {
                let call = downcast_ref::<middle::ir::instruction::misc_inst::Call>(
                    inst.as_ref().as_ref(),
                );
                Self::build_call_inst(call, stack_allocator, stack_slots, reg_gener, regs)
            }
        }
    }

    /// alloca instruction only instruct allocating memory on stack,not generate one-one instruction
    fn build_alloca_inst(
        alloca: &middle::ir::instruction::memory_op_inst::Alloca,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Address, StackSlot>,
    ) -> Result<Vec<Inst>> {
        let ty = alloca.value_type.clone();
        let bits: u32 = match ty {
            middle::ir::ValueType::Int => 4,
            middle::ir::ValueType::Void => {
                return Err(anyhow!("it can't alloca void")).with_context(|| context!())
            }
            middle::ir::ValueType::Float => 4,
            middle::ir::ValueType::Bool => 4,
            middle::ir::ValueType::Array(_, _) => todo!(),
            middle::ir::ValueType::Pointer(_) => todo!(), // 4B
                                                          // _ => todo!(),             // TODO 如果是其他大小的指令
        };
        let ss = stack_allocator.alloc(bits);
        stack_slots.insert(alloca as *const _ as Address, ss);
        Ok(vec![])
    }

    /// store 指令，有几种可能: 指针/数组、全局变量、栈
    pub fn build_store_inst(
        store: &middle::ir::instruction::memory_op_inst::Store,
        stack_slots: &mut HashMap<Address, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &HashMap<Address, Reg>,
    ) -> Result<Vec<Inst>> {
        // 这个 address
        // 有三种来源: 1. 全局变量 2. alloca 3. get_element_ptr
        let address: &&middle::ir::Operand = &store.get_ptr();
        // address 这个地址是 stack 上的地址
        let address = Self::address_from(address, stack_slots).with_context(|| context!())?;
        let val = &store.get_value();
        let val = Self::local_operand_from(val, regs).with_context(|| context!())?;
        let mut ret: Vec<Inst> = Vec::new();
        match val {
            Operand::Imm(imm) => {
                let dst = reg_gener.gen_virtual_usual_reg(); // 分配一个临时的 dest, 用来存储 imm, 因此 sd reg, stack_slot
                let li = AddInst::new(dst.into(), REG_ZERO.into(), imm.into()); // li dst, imm
                let src = dst;
                let sd = StoreInst::new(address.try_into()?, src); // sd src, stack_slot
                ret.push(li.into());
                ret.push(sd.into());
            }
            Operand::Fmm(_) => {
                return Err(anyhow!("store instruction with float value".to_string(),))
                    .with_context(|| context!());
            }
            _ => (),
        }
        Ok(ret)
    }

    pub fn build_load_inst(
        load: &middle::ir::instruction::memory_op_inst::Load,
        stack_slots: &mut HashMap<Address, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &HashMap<Address, Reg>,
    ) -> Result<Vec<Inst>> {
        // dbg!(load);
        let mut ret: Vec<Inst> = Vec::new();
        todo!();
        Ok(ret)
    }

    pub fn build_ret_inst(
        ret: &middle::ir::instruction::terminator_inst::Ret,
        regs: &mut HashMap<Address, Reg>,
    ) -> Result<Vec<Inst>> {
        let mut ret_insts: Vec<Inst> = Vec::new();

        // 准备返回值
        if !ret.is_void() {
            let op = ret.get_return_value();
            match op {
                middle::ir::Operand::Constant(c) => match c {
                    middle::ir::Constant::Int(i) => {
                        let imm: Operand = (*i as i64).into();
                        let li = AddInst::new(REG_A0.into(), REG_ZERO.into(), imm);
                        ret_insts.push(li.into());
                    }
                    middle::ir::Constant::Bool(b) => {
                        let imm: Operand = (*b as i64).into();
                        let li = AddInst::new(REG_A0.into(), REG_ZERO.into(), imm);
                        ret_insts.push(li.into());
                    }
                    middle::ir::Constant::Float(f) => {
                        let fmm = (*f as f64).into();
                        let li = AddInst::new(REG_FA0.into(), REG_ZERO.into(), fmm);
                        ret_insts.push(li.into());
                    }
                    middle::ir::Constant::Array(_) => {
                        return Err(anyhow!("return array is not allow:{}", op))
                            .with_context(|| context!())
                    }
                },
                middle::ir::Operand::Instruction(instr) => {
                    let addr = instr.as_ref().as_ref() as *const dyn middle::ir::Instruction
                        as *const () as Address;
                    let reg = regs.get(&addr).ok_or(anyhow!("").context(context!()))?; // 获取返回值对应的虚拟寄存器
                    let mv_inst = match instr.get_value_type() {
                        middle::ir::ValueType::Int => MvInst::new(REG_A0.into(), (*reg).into()),
                        middle::ir::ValueType::Float => MvInst::new(REG_FA0.into(), (*reg).into()),
                        middle::ir::ValueType::Void => {
                            return Err(anyhow!("return not is_void, but get void type"))
                                .with_context(|| context!())
                        }
                        middle::ir::ValueType::Bool => MvInst::new(REG_A0.into(), (*reg).into()),
                        middle::ir::ValueType::Array(_, _) => {
                            return Err(anyhow!("return array is not allow for sysy"))
                                .with_context(|| context!())
                        }
                        middle::ir::ValueType::Pointer(_) => {
                            // NOTE 注意一下这里 可能可以返回指针
                            return Err(anyhow!("return pointer is not allow for sysy"))
                                .with_context(|| context!());
                        }
                    };
                    ret_insts.push(mv_inst.into());
                }
                middle::ir::Operand::Global(glo) => {
                    return Err(anyhow!("return global should be load first :{}", glo))
                        .with_context(|| context!())
                }
                middle::ir::Operand::Parameter(param) => {
                    let addr = param.as_ref() as *const _ as Address;
                    let reg = regs.get(&addr).ok_or(anyhow!("").context(context!()))?;
                    let mv_inst = match param.value_type {
                        middle::ir::ValueType::Void => {
                            return Err(anyhow!("return not is_void, but get void type"))
                                .with_context(|| context!())
                        }
                        middle::ir::ValueType::Int => MvInst::new(REG_A0.into(), (*reg).into()),
                        middle::ir::ValueType::Float => MvInst::new(REG_FA0.into(), (*reg).into()),
                        middle::ir::ValueType::Bool => MvInst::new(REG_A0.into(), (*reg).into()),
                        middle::ir::ValueType::Array(_, _) => {
                            return Err(anyhow!("return array is not allow for sysy"))
                                .with_context(|| context!())
                        }
                        middle::ir::ValueType::Pointer(_) => {
                            return Err(anyhow!("return pointer is not allow for sysy"))
                                .with_context(|| context!())
                        }
                    };
                    ret_insts.push(mv_inst.into());
                }
            }
        } /*  else {  // 如果返回值是 void, 那么啥也不应管了
          } */

        // 最后的一条 ret
        ret_insts.push(Inst::Ret);
        Ok(ret_insts)
    }

    pub fn build_br_inst(
        br: &middle::ir::instruction::terminator_inst::Br,
        regs: &mut HashMap<Address, Reg>,
        reg_gener: &mut RegGenerator,
    ) -> Result<Vec<Inst>> {
        let cur = br
            .get_parent_bb()
            .ok_or(anyhow!("iffalse get error",))
            .with_context(|| context!())?;

        let succs = cur.get_succ_bb();

        let mut br_insts: Vec<Inst> = Vec::new();
        if br.is_cond_br() {
            // 获取 cond 对应的寄存器
            let reg: Reg = match br.get_cond() {
                // 如果是常数，那么需要使用 li 将常数加载到寄存器中
                middle::ir::Operand::Constant(con) => match con {
                    middle::ir::Constant::Int(i) => {
                        let imm: Operand = (*i as i64).into();
                        let reg = reg_gener.gen_virtual_usual_reg();
                        let li = AddInst::new(reg.into(), REG_ZERO.into(), imm);
                        br_insts.push(li.into());
                        reg
                    }
                    middle::ir::Constant::Bool(bo) => {
                        let imm: Operand = (*bo as i64).into();
                        let reg = reg_gener.gen_virtual_usual_reg();
                        let li = AddInst::new(reg.into(), REG_ZERO.into(), imm);
                        br_insts.push(li.into());
                        reg
                    }
                    _ => {
                        return Err(anyhow!("cond br with array or float is not allow"))
                            .with_context(|| context!())
                    }
                },
                middle::ir::Operand::Parameter(param) => match param.as_ref().value_type {
                    middle::ir::ValueType::Int | middle::ir::ValueType::Bool => {
                        let addr = param.as_ref() as *const _ as Address;
                        let reg = regs.get(&addr).ok_or(anyhow!("").context(context!()))?;
                        *reg
                    }
                    _ => {
                        return Err(anyhow!(
                            "cond br with array/float/pointer/void is not allow"
                        ))
                        .with_context(|| context!())
                    }
                },
                middle::ir::Operand::Instruction(instr) => match instr.get_value_type() {
                    middle::ir::ValueType::Int | middle::ir::ValueType::Bool => {
                        let addr = instr.as_ref().as_ref() as *const dyn middle::ir::Instruction
                            as *const () as Address;
                        let reg = regs.get(&addr).ok_or(anyhow!("").context(context!()))?;
                        *reg
                    }
                    _ => {
                        return Err(anyhow!(
                            "cond br with array/float/pointer/void is not allow"
                        ))
                        .with_context(|| context!())
                    }
                },
                middle::ir::Operand::Global(_) => {
                    return Err(anyhow!("cond br with global is not allow"))
                        .with_context(|| context!())
                }
            };

            let iftrue = succs
                .first()
                .ok_or(anyhow!("iftrue get error",))
                .with_context(|| context!())?;
            let iffalse = succs
                .get(1)
                .ok_or(anyhow!("iftrue get error",))
                .with_context(|| context!())?;

            br_insts.extend(vec![
                Inst::Bne(BneInst::new(
                    reg,
                    REG_ZERO,
                    (iffalse.as_ref() as *const _ as Address).to_string().into(),
                )),
                Inst::Jmp(JmpInst::new(
                    (iftrue.as_ref() as *const _ as Address).to_string().into(),
                )),
            ]);
        } else {
            let succ = succs
                .first()
                .ok_or(anyhow!("iftrue get error",))
                .with_context(|| context!())?;
            br_insts.push(Inst::Jmp(JmpInst::new(
                (succ.as_ref() as *const _ as Address).to_string().into(),
            )))
        }

        Ok(br_insts)
    }

    /// 不是 ret 就是 br
    pub fn build_term_inst(
        term: &ObjPtr<Box<dyn middle::ir::Instruction>>,
        regs: &mut HashMap<Address, Reg>,
        reg_gener: &mut RegGenerator,
    ) -> Result<Vec<Inst>> {
        let mut ret_insts: Vec<Inst> = Vec::new();
        // dbg!(term);

        let insts = match term.get_type() {
            middle::ir::instruction::InstType::Ret => {
                let ret = downcast_ref::<middle::ir::instruction::terminator_inst::Ret>(
                    term.as_ref().as_ref(),
                );
                Self::build_ret_inst(ret, regs)?
            }
            middle::ir::instruction::InstType::Br => {
                let br = downcast_ref::<middle::ir::instruction::terminator_inst::Br>(
                    term.as_ref().as_ref(),
                );
                Self::build_br_inst(br, regs, reg_gener)?
            }
            _ => {
                return Err(anyhow!("get_last_inst only to be ret or br"))
                    .with_context(|| context!())
            }
        };

        // TODO 这里 return 还要记得 退栈

        ret_insts.extend(insts);

        Ok(ret_insts)
    }

    pub fn build_call_inst(
        // call: &llvm_ir::instruction::Call,
        call: &middle::ir::instruction::misc_inst::Call,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Address, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
    ) -> Result<Vec<Inst>> {
        let mut call_insts: Vec<Inst> = Vec::new();

        let params = call.get_operand(); // 参数列表

        // 函数是全局的，因此用的是名字
        let call_inst = CallInst::new(call.func.name.to_string().into()).into(); // call <label>
        call_insts.push(call_inst);

        let dest_name = call as *const _ as Address;

        let func = call.func;

        // call 返回之后，将返回值放到一个虚拟寄存器中
        match func.return_type {
            middle::ir::ValueType::Void => {}
            middle::ir::ValueType::Int => {
                let dst = reg_gener.gen_virtual_usual_reg(); // 分配一个虚拟寄存器
                let mv = MvInst::new(dst.into(), REG_A0.into());
                call_insts.push(mv.into()); // 插入 mv 指令
                regs.insert(dest_name, dst); // 绑定中端的 id 和 虚拟寄存器
            }
            middle::ir::ValueType::Float => {
                let dst = reg_gener.gen_virtual_float_reg();
                let mv = MvInst::new(dst.into(), REG_FA0.into());
                call_insts.push(mv.into());
                regs.insert(dest_name, dst);
            }
            middle::ir::ValueType::Bool => {
                let dst = reg_gener.gen_virtual_usual_reg();
                let mv = MvInst::new(dst.into(), REG_A0.into());
                call_insts.push(mv.into());
                regs.insert(dest_name, dst);
            }
            _ => {
                return Err(anyhow!("sysy only return: void | float | int".to_string()))
                    .with_context(|| context!())
            }
        };

        Ok(call_insts)
    }
}

mod tests {
    #[allow(unused)]
    use crate::{
        backend::from::from_self::Address,
        middle::{
            self,
            ir::{instruction::downcast_ref, IRBuilder, Instruction, ValueType},
        },
    };

    /// 测试地址是否改变
    #[test]
    fn test_address() {
        let mut ir_builder = IRBuilder::new();
        let ptr = ir_builder.get_alloca(ValueType::Int, 1);
        let load_0 = ir_builder.get_load(ValueType::Int, middle::ir::Operand::Instruction(ptr));

        let ss = load_0.as_ref().as_ref() as *const dyn Instruction as *const () as Address;
        dbg!(&ss);

        let inst: &middle::ir::instruction::memory_op_inst::Load =
            downcast_ref::<middle::ir::instruction::memory_op_inst::Load>(load_0.as_ref().as_ref());

        let address = inst as *const middle::ir::instruction::memory_op_inst::Load as Address;
        dbg!(&address);

        after_downcast(inst);
    }

    #[allow(dead_code)]
    fn after_downcast(inst: &middle::ir::instruction::memory_op_inst::Load) {
        let address = inst as *const middle::ir::instruction::memory_op_inst::Load as Address;
        dbg!(&address);
    }
}
