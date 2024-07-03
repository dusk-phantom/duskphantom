use crate::backend::*;
use crate::utils::mem::ObjPtr;
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
            middle::ir::instruction::InstType::Head => {
                Err(anyhow!("head should not be in backend")).with_context(|| context!())
            } // 应该是不能有 Head 出现的
            middle::ir::instruction::InstType::Add => {
                let add = downcast_ref::<middle::ir::instruction::binary_inst::Add>(
                    inst.as_ref().as_ref(),
                );
                let lhs =
                    Self::local_operand_from(add.get_lhs(), regs).with_context(|| context!())?;
                let rhs =
                    Self::local_operand_from(add.get_rhs(), regs).with_context(|| context!())?;

                let dst = reg_gener.gen_virtual_usual_reg();
                regs.insert(add as *const _ as Address, dst);
                let inst = AddInst::new(dst.into(), lhs, rhs);
                Ok(vec![Inst::Add(inst)])
            }
            middle::ir::instruction::InstType::FAdd => {
                let add = downcast_ref::<middle::ir::instruction::binary_inst::FAdd>(
                    inst.as_ref().as_ref(),
                );
                let lhs =
                    Self::local_operand_from(add.get_lhs(), regs).with_context(|| context!())?;
                let rhs =
                    Self::local_operand_from(add.get_rhs(), regs).with_context(|| context!())?;

                let dst = reg_gener.gen_virtual_usual_reg();
                regs.insert(add as *const _ as Address, dst);
                let inst = AddInst::new(dst.into(), lhs, rhs);
                Ok(vec![Inst::Add(inst)])
            }
            middle::ir::instruction::InstType::Sub => {
                let sub = downcast_ref::<middle::ir::instruction::binary_inst::Sub>(
                    inst.as_ref().as_ref(),
                );
                let lhs =
                    Self::local_operand_from(sub.get_lhs(), regs).with_context(|| context!())?;
                let rhs =
                    Self::local_operand_from(sub.get_rhs(), regs).with_context(|| context!())?;
                let dst = reg_gener.gen_virtual_usual_reg();
                regs.insert(sub as *const _ as Address, dst);
                let inst = SubInst::new(dst.into(), lhs, rhs);
                Ok(vec![Inst::Sub(inst)])
            }
            middle::ir::instruction::InstType::FSub => todo!(),
            middle::ir::instruction::InstType::Mul => {
                let mul = downcast_ref::<middle::ir::instruction::binary_inst::Mul>(
                    inst.as_ref().as_ref(),
                );
                let lhs =
                    Self::local_operand_from(mul.get_lhs(), regs).with_context(|| context!())?;
                let rhs =
                    Self::local_operand_from(mul.get_rhs(), regs).with_context(|| context!())?;
                let dst = reg_gener.gen_virtual_usual_reg();
                regs.insert(mul as *const _ as Address, dst);
                let inst = MulInst::new(dst.into(), lhs, rhs);
                Ok(vec![Inst::Mul(inst)])
            }
            middle::ir::instruction::InstType::FMul => todo!(),
            middle::ir::instruction::InstType::UDiv => {
                todo!();
                // let mul = downcast_ref::<middle::ir::instruction::binary_inst::Mul>(
                //     inst.as_ref().as_ref(),
                // );
                // let lhs =
                //     Self::local_operand_from(mul.get_lhs(), regs).with_context(|| context!())?;
                // let rhs =
                //     Self::local_operand_from(mul.get_rhs(), regs).with_context(|| context!())?;
                // let dst = reg_gener.gen_virtual_usual_reg();
                // let inst = MulInst::new(dst.into(), lhs, rhs);
                // Ok(vec![Inst::Mul(inst)])
            }
            middle::ir::instruction::InstType::SDiv => todo!(),
            middle::ir::instruction::InstType::FDiv => todo!(),
            middle::ir::instruction::InstType::URem => todo!(),
            middle::ir::instruction::InstType::SRem => todo!(),
            middle::ir::instruction::InstType::Shl => {
                let shl = downcast_ref::<middle::ir::instruction::binary_inst::Shl>(
                    inst.as_ref().as_ref(),
                );
                let lhs =
                    Self::local_operand_from(shl.get_lhs(), regs).with_context(|| context!())?;
                let rhs =
                    Self::local_operand_from(shl.get_rhs(), regs).with_context(|| context!())?;
                let dst = reg_gener.gen_virtual_usual_reg();
                regs.insert(shl as *const _ as Address, dst);
                let inst = SllInst::new(dst.into(), lhs, rhs);
                Ok(vec![Inst::Sll(inst)])
            }
            middle::ir::instruction::InstType::LShr => {
                let lshr = downcast_ref::<middle::ir::instruction::binary_inst::LShr>(
                    inst.as_ref().as_ref(),
                );
                let lhs =
                    Self::local_operand_from(lshr.get_lhs(), regs).with_context(|| context!())?;
                let rhs =
                    Self::local_operand_from(lshr.get_rhs(), regs).with_context(|| context!())?;
                let dst = reg_gener.gen_virtual_usual_reg();
                regs.insert(lshr as *const _ as Address, dst);
                let inst = SrlInst::new(dst.into(), lhs, rhs);
                Ok(vec![Inst::Srl(inst)])
            }
            middle::ir::instruction::InstType::AShr => {
                let ashr = downcast_ref::<middle::ir::instruction::binary_inst::AShr>(
                    inst.as_ref().as_ref(),
                );
                let lhs =
                    Self::local_operand_from(ashr.get_lhs(), regs).with_context(|| context!())?;
                let rhs =
                    Self::local_operand_from(ashr.get_rhs(), regs).with_context(|| context!())?;
                let dst = reg_gener.gen_virtual_usual_reg();
                regs.insert(ashr as *const _ as Address, dst);
                let inst = SraInst::new(dst.into(), lhs, rhs);
                Ok(vec![Inst::SRA(inst)])
            }
            middle::ir::instruction::InstType::And => {
                let and = downcast_ref::<middle::ir::instruction::binary_inst::And>(
                    inst.as_ref().as_ref(),
                );
                let lhs =
                    Self::local_operand_from(and.get_lhs(), regs).with_context(|| context!())?;
                let rhs =
                    Self::local_operand_from(and.get_rhs(), regs).with_context(|| context!())?;
                let dst = reg_gener.gen_virtual_usual_reg();
                regs.insert(and as *const _ as Address, dst);
                let inst = AndInst::new(dst.into(), lhs, rhs);
                Ok(vec![Inst::And(inst)])
            }
            middle::ir::instruction::InstType::Or => {
                let or = downcast_ref::<middle::ir::instruction::binary_inst::Or>(
                    inst.as_ref().as_ref(),
                );
                let lhs =
                    Self::local_operand_from(or.get_lhs(), regs).with_context(|| context!())?;
                let rhs =
                    Self::local_operand_from(or.get_rhs(), regs).with_context(|| context!())?;
                let dst = reg_gener.gen_virtual_usual_reg();
                regs.insert(or as *const _ as Address, dst);
                let inst = OrInst::new(dst.into(), lhs, rhs);
                Ok(vec![Inst::Or(inst)])
            }
            middle::ir::instruction::InstType::Xor => {
                let xor = downcast_ref::<middle::ir::instruction::binary_inst::Xor>(
                    inst.as_ref().as_ref(),
                );
                let lhs =
                    Self::local_operand_from(xor.get_lhs(), regs).with_context(|| context!())?;
                let rhs =
                    Self::local_operand_from(xor.get_rhs(), regs).with_context(|| context!())?;
                let dst = reg_gener.gen_virtual_usual_reg();
                regs.insert(xor as *const _ as Address, dst);
                let inst = XorInst::new(dst.into(), lhs, rhs);
                Ok(vec![Inst::Xor(inst)])
            }
            middle::ir::instruction::InstType::Ret => {
                let ret = downcast_ref::<middle::ir::instruction::terminator_inst::Ret>(
                    inst.as_ref().as_ref(),
                );
                let lhs = ret.get_return_value();
                todo!();
                Self::build_ret_inst(ret, regs)
            }
            middle::ir::instruction::InstType::Br => todo!(),
            middle::ir::instruction::InstType::Load => todo!(),
            middle::ir::instruction::InstType::GetElementPtr => todo!(),
            middle::ir::instruction::InstType::ZextTo => todo!(),
            middle::ir::instruction::InstType::SextTo => todo!(),
            middle::ir::instruction::InstType::ItoFp => todo!(),
            middle::ir::instruction::InstType::FpToI => todo!(),
            middle::ir::instruction::InstType::ICmp => todo!(),
            middle::ir::instruction::InstType::FCmp => todo!(),
            middle::ir::instruction::InstType::Phi => todo!(),
            middle::ir::instruction::InstType::Call => todo!(),
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

    #[allow(unused)]
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

    /// TODO 包含最后的 ret 语句
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

    /// 不是 ret 就是 br
    pub fn build_term_inst(
        term: &ObjPtr<Box<dyn middle::ir::Instruction>>,
        regs: &mut HashMap<Address, Reg>,
    ) -> Result<Vec<Inst>> {
        let mut ret_insts: Vec<Inst> = Vec::new();
        // dbg!(term);

        match term.get_type() {
            middle::ir::instruction::InstType::Ret => {
                let ret = downcast_ref::<middle::ir::instruction::terminator_inst::Ret>(
                    term.as_ref().as_ref(),
                );
            }
            middle::ir::instruction::InstType::Br => {
                todo!();
            }
            _ => {
                return Err(anyhow!("get_last_inst only to be ret or br"))
                    .with_context(|| context!())
            }
        }

        Ok(ret_insts)
    }

    #[allow(unused)]
    pub fn build_call_inst(
        // call: &llvm_ir::instruction::Call,
        call: &middle::ir::instruction::misc_inst::Call,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Address, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
    ) -> Result<Vec<Inst>> {
        let mut ret: Vec<Inst> = Vec::new();

        let f_name = &call.func.name; // 要跳转的函数名
        let params = call.get_operand(); // 参数列表

        // 调用惯例

        let call_inst = CallInst::new(f_name.to_string().into()).into(); // call <label>
        ret.push(call_inst);

        let dest_name = call as *const _ as Address;

        let func = call.func;

        match func.return_type {
            middle::ir::ValueType::Void => todo!(),
            middle::ir::ValueType::Int => {
                let dst = reg_gener.gen_virtual_usual_reg(); // 分配一个虚拟寄存器
                let mv = MvInst::new(dst.into(), REG_A0.into());
                ret.push(mv.into()); // 插入 mv 指令
                regs.insert(dest_name, dst); // 绑定中端的 id 和 虚拟寄存器
            }
            middle::ir::ValueType::Float => {
                let dst = reg_gener.gen_virtual_float_reg();
                let mv = MvInst::new(dst.into(), REG_FA0.into());
                ret.push(mv.into());
                regs.insert(dest_name, dst);
            }
            middle::ir::ValueType::Bool => {
                let dst = reg_gener.gen_virtual_usual_reg();
                let mv = MvInst::new(dst.into(), REG_A0.into());
                ret.push(mv.into());
                regs.insert(dest_name, dst);
            }
            _ => {
                return Err(anyhow!("sysy only return: void | float | int".to_string()))
                    .with_context(|| context!())
            }
        };

        Ok(ret)
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
