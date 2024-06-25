use crate::backend::*;
use crate::middle::ir::instruction::memory_op_inst::{Alloca, Store};
use crate::middle::ir::instruction::{downcast_ref, InstType};
use crate::middle::ir::{Instruction, ValueType};
use crate::utils::mem::ObjPtr;
use crate::{context, middle};

use super::*;

use anyhow::{Context, Result};

use anyhow::Ok;
use builder::IRBuilder;
use llvm_ir::Name;
use std::any::Any;
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
            InstType::Add => {
                todo!();
            }
            InstType::FAdd => {
                todo!();
            }
            InstType::Sub => {
                todo!();
            }
            InstType::FSub => {
                todo!();
            }
            InstType::Mul => {
                todo!();
            }
            InstType::FMul => {
                todo!();
            }
            InstType::UDiv => {
                todo!();
            }
            InstType::SDiv => {
                todo!();
            }
            InstType::FDiv => {
                todo!();
            }
            InstType::URem => {
                todo!();
            }
            InstType::SRem => {
                todo!();
            }
            InstType::Shl => {
                todo!();
            }
            InstType::LShr => {
                todo!();
            }
            InstType::AShr => {
                todo!();
            }
            InstType::And => {
                todo!();
            }
            InstType::Or => {
                todo!();
            }
            InstType::Xor => todo!(),
            InstType::Ret => {
                todo!();
            }
            InstType::Br => {
                todo!();
            }
            InstType::Alloca => {
                let alloca = downcast_ref::<Alloca>(inst.as_ref().as_ref());
                Self::build_alloca_inst(alloca, stack_allocator, stack_slots)
            }
            InstType::Load => {
                todo!();
            }
            InstType::Store => {
                let store = downcast_ref::<Store>(inst.as_ref().as_ref());
                Self::build_store_inst(store, stack_slots, reg_gener, regs)
            }
            InstType::GetElementPtr => {
                todo!();
            }
            InstType::ZextTo => {
                todo!();
            }
            InstType::SextTo => {
                todo!();
            }
            InstType::ItoFp => {
                todo!();
            }
            InstType::FpToI => {
                todo!();
            }
            InstType::ICmp => {
                todo!();
            }
            InstType::FCmp => {
                todo!();
            }
            InstType::Phi => {
                todo!();
            }
            InstType::Call => {
                todo!();
            }
            _ => {
                unreachable!()
            }
        }
    }

    /// alloca instruction only instruct allocating memory on stack,not generate one-one instruction
    fn build_alloca_inst(
        alloca: &Alloca,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Name, StackSlot>,
    ) -> Result<Vec<Inst>> {
        let name = alloca.get_id();
        let ty = alloca.value_type.clone();
        let bits = match ty {
            ValueType::Int => 4usize, // 4B
            _ => todo!(),             // TODO
        };
        let ss = stack_allocator.alloc(bits as usize);
        stack_slots.insert(name.into(), ss);
        Ok(vec![])
    }

    pub fn build_store_inst(
        store: &Store,
        stack_slots: &mut HashMap<Name, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        let address = &store.get_ptr();
        let val = &store.get_value();
        let address = Self::address_from(address, stack_slots).with_context(|| context!())?;
        let val = Self::value_from(val, regs).with_context(|| context!())?;
        dbg!("gg");
        let mut ret: Vec<Inst> = Vec::new();
        match val {
            Operand::Imm(imm) => {
                let dst = reg_gener.gen_virtual_usual_reg();
                let li = AddInst::new(dst.into(), REG_ZERO.into(), imm.into());
                let sd = StoreInst::new(dst, address.try_into()?);
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
        term: &middle::ir::instruction::terminator_inst::Ret,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        let mut ret_insts: Vec<Inst> = Vec::new();
        // dbg!(term);

        // match term {
        //     llvm_ir::Terminator::Ret(r) => {
        //         if let Some(op) = &r.return_operand {
        //             match op {
        //                 llvm_ir::Operand::LocalOperand { name, ty } => {
        //                     let reg = regs.get(name).ok_or(anyhow!("").context(context!()))?;
        //                     let mv_inst = match ty.as_ref() {
        //                         llvm_ir::Type::IntegerType { bits: _ } => {
        //                             MvInst::new(REG_A0.into(), reg.into())
        //                         }
        //                         llvm_ir::Type::FPType(_) => MvInst::new(REG_FA0.into(), reg.into()),
        //                         _ => unimplemented!(),
        //                     };
        //                     ret_insts.push(mv_inst.into());
        //                     ret_insts.push(Inst::Ret);
        //                 }
        //                 llvm_ir::Operand::ConstantOperand(c) => match c.as_ref() {
        //                     Constant::Int { bits: _, value } => {
        //                         let imm = (*value as i64).into();
        //                         let addi = AddInst::new(REG_A0.into(), REG_ZERO.into(), imm);
        //                         ret_insts.push(addi.into());
        //                         ret_insts.push(Inst::Ret);
        //                     }
        //                     Constant::Float(_) => todo!(),
        //                     _ => todo!(),
        //                 },
        //                 llvm_ir::Operand::MetadataOperand => todo!(),
        //             }
        //         } else {
        //             unimplemented!();
        //         }
        //     }
        //     _ => todo!(),
        // }
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

        unimplemented!();

        // // let func_ty = &call.type_id();
        // let dst_reg: Reg = match func_ty.as_ref() {
        //     llvm_ir::Type::FuncType {
        //         result_type,
        //         param_types,
        //         is_var_arg,
        //     } => match result_type.as_ref() {
        //         llvm_ir::Type::FPType(_) => {
        //             let dst = reg_gener.gen_virtual_float_reg();
        //             let mv = MvInst::new(dst.into(), REG_FA0.into());
        //             ret.push(mv.into());
        //             dst
        //         }
        //         llvm_ir::Type::IntegerType { bits } => {
        //             let dst = reg_gener.gen_virtual_usual_reg();
        //             let mv = MvInst::new(dst.into(), REG_A0.into());
        //             ret.push(mv.into());
        //             dst
        //         }
        //         _ => {
        //             unimplemented!();
        //         }
        //     },
        //     _ => {
        //         unimplemented!("function type");
        //     }
        // };
        // regs.insert(dest.clone(), dst_reg);
        // // FIXME: process arguments
        // // unimplemented!("process arguments");

        Ok(ret)
    }
}
