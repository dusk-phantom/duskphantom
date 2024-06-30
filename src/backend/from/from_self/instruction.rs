use crate::backend::*;
use crate::utils::mem::ObjPtr;
use crate::{context, middle};

use crate::middle::ir::instruction::downcast_ref;
use crate::middle::ir::{Instruction, ValueType};

use super::*;

use anyhow::{Context, Result};

use anyhow::Ok;
use builder::IRBuilder;
use llvm_ir::Name;
use std::collections::HashMap;

impl IRBuilder {
    pub fn build_instruction(
        inst: &ObjPtr<Box<dyn Instruction>>,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Name, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
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
            middle::ir::instruction::InstType::Head => unreachable!(), // 应该是不能有 Head 出现的
            middle::ir::instruction::InstType::Add => todo!(),
            middle::ir::instruction::InstType::FAdd => todo!(),
            middle::ir::instruction::InstType::Sub => todo!(),
            middle::ir::instruction::InstType::FSub => todo!(),
            middle::ir::instruction::InstType::Mul => todo!(),
            middle::ir::instruction::InstType::FMul => todo!(),
            middle::ir::instruction::InstType::UDiv => todo!(),
            middle::ir::instruction::InstType::SDiv => todo!(),
            middle::ir::instruction::InstType::FDiv => todo!(),
            middle::ir::instruction::InstType::URem => todo!(),
            middle::ir::instruction::InstType::SRem => todo!(),
            middle::ir::instruction::InstType::Shl => todo!(),
            middle::ir::instruction::InstType::LShr => todo!(),
            middle::ir::instruction::InstType::AShr => todo!(),
            middle::ir::instruction::InstType::And => todo!(),
            middle::ir::instruction::InstType::Or => todo!(),
            middle::ir::instruction::InstType::Xor => todo!(),
            middle::ir::instruction::InstType::Ret => todo!(),
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
        stack_slots: &mut HashMap<Name, StackSlot>,
    ) -> Result<Vec<Inst>> {
        let name = alloca.get_id();
        let ty = alloca.value_type.clone();
        let bits = match ty {
            ValueType::Int => 4usize, // 4B
            _ => todo!(),             // TODO 如果是其他大小的指令
        };
        let ss = stack_allocator.alloc(bits as u32);
        stack_slots.insert(name.into(), ss);
        Ok(vec![])
    }

    pub fn build_store_inst(
        store: &middle::ir::instruction::memory_op_inst::Store,
        stack_slots: &mut HashMap<Name, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        let address = &store.get_ptr();
        let val = &store.get_value();
        let address = Self::address_from(address, stack_slots).with_context(|| context!())?;
        let val = Self::value_from(val, regs).with_context(|| context!())?;
        let mut ret: Vec<Inst> = Vec::new();
        match val {
            Operand::Imm(imm) => {
                let dst = reg_gener.gen_virtual_usual_reg();
                let li = AddInst::new(dst.into(), REG_ZERO.into(), imm.into());
                let src = dst;
                let sd = StoreInst::new(address.try_into()?, src);
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
        stack_slots: &mut HashMap<Name, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        // dbg!(load);
        let mut ret: Vec<Inst> = Vec::new();
        todo!();
        Ok(ret)
    }

    pub fn build_term_inst(
        term: &ObjPtr<Box<dyn Instruction>>,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        let mut ret_insts: Vec<Inst> = Vec::new();
        // dbg!(term);

        match term.get_type() {
            middle::ir::instruction::InstType::Ret => {
                let ret = downcast_ref::<middle::ir::instruction::terminator_inst::Ret>(
                    term.as_ref().as_ref(),
                );
                if !ret.is_void() {
                    let op = ret.get_return_value();
                    match op {
                        middle::ir::Operand::Constant(c) => match c {
                            middle::ir::Constant::Int(value) => {
                                let imm = (*value as i64).into();
                                let addi = AddInst::new(REG_A0.into(), REG_ZERO.into(), imm);
                                ret_insts.push(addi.into());
                                ret_insts.push(Inst::Ret);
                            }
                            middle::ir::Constant::Float(_) => todo!(),
                            middle::ir::Constant::Bool(_) => todo!(),
                            middle::ir::Constant::Array(_) => todo!(),
                        },
                        middle::ir::Operand::Instruction(instr) => {
                            let name: Name = instr.get_id().into();
                            let reg = regs.get(&name).ok_or(anyhow!("").context(context!()))?;
                            let mv_inst = match instr.get_value_type() {
                                ValueType::Int => MvInst::new(REG_A0.into(), (*reg).into()),
                                ValueType::Float => unimplemented!(),
                                _ => todo!(),
                            };
                            ret_insts.push(mv_inst.into());
                            ret_insts.push(Inst::Ret);
                        }
                        _ => unreachable!(),
                    }
                } else {
                    unimplemented!();
                }
            }
            middle::ir::instruction::InstType::Br => {
                todo!();
            }
            _ => {
                unreachable!();
            }
        }

        Ok(ret_insts)
    }

    #[allow(unused)]
    pub fn build_call_inst(
        // call: &llvm_ir::instruction::Call,
        call: &middle::ir::instruction::misc_inst::Call,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Name, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        // let dst = &call.dest;
        let dst = &call.get_id();
        let f_name = &call.func.name;
        let mut ret: Vec<Inst> = Vec::new();
        let call_inst = CallInst::new(f_name.to_string().into()).into();
        ret.push(call_inst);

        let dest_name = call.get_id();

        let func = call.func;
        let dst_reg: Reg = match func.return_type {
            ValueType::Void => todo!(),
            ValueType::Int => {
                let dst = reg_gener.gen_virtual_usual_reg();
                let mv = MvInst::new(dst.into(), REG_A0.into());
                ret.push(mv.into());
                dst
            }
            ValueType::Float => {
                let dst = reg_gener.gen_virtual_float_reg();
                let mv = MvInst::new(dst.into(), REG_FA0.into());
                ret.push(mv.into());
                dst
            }
            ValueType::Bool => todo!(),
            ValueType::Array(_, _) => todo!(),
            ValueType::Pointer(_) => todo!(),
        };
        regs.insert(dest_name.into(), dst_reg);

        Ok(ret)
    }
}
