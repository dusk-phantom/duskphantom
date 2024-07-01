use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};

use crate::backend::{Operand, Reg, StackSlot};

use crate::context;

use crate::middle;

use super::*;
use builder::IRBuilder;

use llvm_ir::Name;

impl IRBuilder {
    pub fn is_ty_int(ty: &middle::ir::ValueType) -> bool {
        matches!(ty, middle::ir::ValueType::Int)
    }
    pub fn is_ty_float(ty: &middle::ir::ValueType) -> bool {
        matches!(ty, middle::ir::ValueType::Float)
    }

    pub fn address_from(
        operand: &middle::ir::Operand,
        stack_slots: &HashMap<Name, StackSlot>,
    ) -> Result<Operand> {
        Ok(match operand {
            middle::ir::Operand::Constant(_) => todo!(),
            // TODO store 中的 dst 可能是 全局变量
            middle::ir::Operand::Global(_) => todo!(),
            middle::ir::Operand::Parameter(_) => todo!(),
            // 来源于 get_element_ptr 或者是 alloca
            middle::ir::Operand::Instruction(instr) => stack_slots
                .get(&instr.get_id().into())
                .ok_or(anyhow!("stack slot not found {}", instr.get_id()))
                .with_context(|| context!())?
                .into(), // 这个 into 将 stackslot -> operand
        })
    }
    pub fn local_var_from(
        operand: &middle::ir::Operand,
        regs: &HashMap<Name, Reg>,
    ) -> Result<Operand> {
        Ok(match operand {
            middle::ir::Operand::Instruction(instr) => {
                let reg = regs
                    .get(&instr.get_id().into())
                    .ok_or(anyhow!("local var not found {}", instr.get_id()))
                    .with_context(|| context!())?;
                reg.into()
            }
            _ => {
                return Err(anyhow!("operand is not local var:{}", operand))
                    .with_context(|| context!());
            }
        })
    }

    pub fn float_operand_from(
        operand: &middle::ir::Operand,
        float_regs: &HashMap<Name, Reg>,
    ) -> Result<Operand> {
        match operand {
            middle::ir::Operand::Constant(con) => {
                if let middle::ir::Constant::Float(f) = con {
                    Ok(Operand::Fmm((*f as f64).into()))
                } else {
                    Err(anyhow!("float-type inst, but receive imm"))
                }
            }
            middle::ir::Operand::Instruction(instr) => {
                let name: Name = instr.get_id().into();
                let freg = float_regs
                    .get(&name)
                    .ok_or(anyhow!("").context(context!()))?;
                Ok((*freg).into())
            }
            _ => Err(anyhow!(
                "float-type inst, but receive not neither reg nor fmm"
            )),
        }
    }

    pub fn int_operand_from(
        operand: &middle::ir::Operand,
        usual_regs: &HashMap<Name, Reg>,
    ) -> Result<Operand> {
        match operand {
            middle::ir::Operand::Constant(con) => {
                if let middle::ir::Constant::Int(i) = con {
                    Ok(Operand::Imm((*i as i64).into()))
                } else {
                    Err(anyhow!("int type add inst, but receive fmm"))
                }
            }
            middle::ir::Operand::Instruction(instr) => {
                let name: Name = instr.get_id().into();
                let ireg = usual_regs
                    .get(&name)
                    .ok_or(anyhow!("").context(context!()))?;
                Ok((*ireg).into())
            }
            _ => Err(anyhow!(
                "int type add inst, but receive not neither reg nor imm"
            )),
        }
    }

    pub fn const_from(operand: &middle::ir::Operand) -> Result<Operand> {
        Ok(match operand {
            middle::ir::Operand::Constant(con) => match con {
                middle::ir::Constant::Int(val) => Operand::Imm((*val as i64).into()),
                middle::ir::Constant::Float(fla) => Operand::Fmm((*fla as f64).into()),
                _ => todo!(),
            },
            _ => {
                return Err(anyhow!("operand is not constant:{}", operand))
                    .with_context(|| context!())
            }
        })
    }

    pub fn value_from(operand: &middle::ir::Operand, regs: &HashMap<Name, Reg>) -> Result<Operand> {
        if let Ok(c) = Self::const_from(operand) {
            Ok(c)
        } else if let Ok(c) = Self::local_var_from(operand, regs) {
            Ok(c)
        } else {
            Err(anyhow!("value neither is reg or const:{}", operand)).with_context(|| context!())
        }
    }

    #[allow(unused)]
    #[inline]
    pub fn global_name_from(operand: &middle::ir::Operand) -> Result<Name> {
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
