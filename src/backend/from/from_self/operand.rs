use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};

use crate::backend::{Operand, Reg, StackSlot};

use crate::context;

use crate::middle;
use crate::middle::ir::Instruction;
use crate::utils::mem::ObjPtr;

use super::*;
use builder::IRBuilder;

impl IRBuilder {
    pub fn is_ty_int(ty: &middle::ir::ValueType) -> bool {
        matches!(ty, middle::ir::ValueType::Int)
    }
    pub fn is_ty_float(ty: &middle::ir::ValueType) -> bool {
        matches!(ty, middle::ir::ValueType::Float)
    }

    pub fn address_from(
        operand: &middle::ir::Operand,
        stack_slots: &HashMap<Address, StackSlot>,
    ) -> Result<Operand> {
        Ok(match operand {
            middle::ir::Operand::Constant(_) => todo!(),
            // TODO store 中的 dst 可能是 全局变量
            middle::ir::Operand::Global(_) => todo!(),
            middle::ir::Operand::Parameter(_) => todo!(),
            // 来源于 get_element_ptr 或者是 alloca
            middle::ir::Operand::Instruction(instr) => stack_slots
                .get(&(instr.as_ref().as_ref() as *const dyn Instruction as *const () as Address))
                .ok_or(anyhow!(
                    "stack slot not found {}",
                    (instr.as_ref().as_ref() as *const dyn Instruction as *const () as Address)
                ))
                .with_context(|| context!())?
                .into(), // 这个 into 将 stackslot -> operand
        })
    }

    pub fn local_var_from(
        instr: &ObjPtr<Box<dyn Instruction>>,
        regs: &HashMap<Address, Reg>,
    ) -> Result<Operand> {
        let addr = instr.as_ref().as_ref() as *const dyn Instruction as *const () as Address;
        let reg = regs
            .get(&addr)
            .ok_or(anyhow!(
                "local var not found {}",
                instr.as_ref().as_ref() as *const dyn Instruction as *const () as Address
            ))
            .with_context(|| context!())?;
        Ok((*reg).into())
    }
    pub fn const_from(con: &middle::ir::Constant) -> Result<Operand> {
        Ok(match con {
            middle::ir::Constant::Int(val) => Operand::Imm((*val as i64).into()),
            middle::ir::Constant::Float(fla) => Operand::Fmm((*fla as f64).into()),
            middle::ir::Constant::Bool(boo) => Operand::Imm((*boo as i64).into()),
            middle::ir::Constant::Array(_) => {
                return Err(anyhow!("const_from operand cann't not be array:{}", con))
                    .with_context(|| context!())
            }
        })
    }
    pub fn parameter_from(
        param: &ObjPtr<middle::ir::Parameter>,
        regs: &HashMap<Address, Reg>,
    ) -> Result<Operand> {
        let addr = param.as_ref() as *const _ as Address;
        let reg = regs
            .get(&addr)
            .ok_or(anyhow!(
                "local var not found {}",
                param.as_ref() as *const _ as Address
            ))
            .with_context(|| context!())?;
        Ok((*reg).into())
    }

    /// 要不是 instruction 的输出, 要不是 constant
    pub fn local_operand_from(
        operand: &middle::ir::Operand,
        regs: &HashMap<Address, Reg>,
    ) -> Result<Operand> {
        match operand {
            middle::ir::Operand::Constant(con) => Self::const_from(con),
            middle::ir::Operand::Parameter(param) => Self::parameter_from(param, regs),
            middle::ir::Operand::Instruction(instr) => Self::local_var_from(instr, regs),
            middle::ir::Operand::Global(glo) => Err(anyhow!(
                "local_operand_from operand cann't not be global:{}",
                glo
            ))
            .with_context(|| context!()),
        }
    }

    #[allow(unused)]
    #[inline]
    pub fn global_name_from(operand: &middle::ir::Operand) -> Result<Address> {
        unimplemented!();
        match operand {
            // middle::ir::Operand::LocalOperand { name: _, ty: _ } => {
            //     Err(anyhow!("local operand".to_string())).with_context(|| context!())
            // }
            middle::ir::Operand::Constant(con) => match con {
                // Constant::GlobalReference { name, ty: _ } => Ok(name.clone()),
                // Ok(llvm_ir::Name::from(glo.name))
                // middle::ir::Operand::Global(glo) => todo!(),
                middle::ir::Constant::Int(_) => todo!(),
                middle::ir::Constant::Float(_) => todo!(),
                middle::ir::Constant::Bool(_) => todo!(),
                middle::ir::Constant::Array(_) => todo!(),
                _ => todo!(),
            },
            middle::ir::Operand::Instruction(instr) => {
                Err(anyhow!("local operand".to_string())).with_context(|| context!())
            }
            middle::ir::Operand::Global(_) => todo!(),
            middle::ir::Operand::Parameter(_) => todo!(),
        }
    }
}
