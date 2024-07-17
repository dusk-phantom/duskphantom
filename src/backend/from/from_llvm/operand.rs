use std::collections::HashMap;

use super::*;
use builder::IRBuilder;

use llvm_ir::{Constant, Name};

impl TryFrom<&llvm_ir::constant::Float> for Fmm {
    type Error = anyhow::Error;
    fn try_from(value: &llvm_ir::constant::Float) -> std::result::Result<Self, Self::Error> {
        match value {
            llvm_ir::constant::Float::Single(f) => Ok(f.into()),
            llvm_ir::constant::Float::Double(f) => Ok(f.into()),
            _ => Err(anyhow!("float type not supported").context(context!())),
        }
    }
}

impl IRBuilder {
    pub fn is_ty_int(ty: &llvm_ir::Type) -> bool {
        matches!(ty, llvm_ir::Type::IntegerType { bits: _ })
    }
    pub fn is_ty_float(ty: &llvm_ir::Type) -> bool {
        matches!(ty, llvm_ir::Type::FPType(_))
    }
    pub fn is_ty_void(ty: &llvm_ir::Type) -> bool {
        matches!(ty, llvm_ir::Type::VoidType)
    }
    pub fn new_var(ty: &llvm_ir::Type, reg_gener: &mut RegGenerator) -> Result<Reg> {
        let dst_reg = if Self::is_ty_int(ty) {
            reg_gener.gen_virtual_usual_reg()
        } else if Self::is_ty_float(ty) {
            reg_gener.gen_virtual_float_reg()
        } else {
            unimplemented!();
        };
        Ok(dst_reg)
    }

    pub fn stack_slot_from(
        operand: &llvm_ir::Operand,
        stack_slots: &HashMap<Name, StackSlot>,
    ) -> Result<Operand> {
        Ok(match operand {
            llvm_ir::Operand::LocalOperand { name, ty: _ } => stack_slots
                .get(name)
                .ok_or(anyhow!("stack slot not found {}", name))
                .with_context(|| context!())?
                .into(),
            _ => {
                return Err(anyhow!("operand is not local var:{}", operand))
                    .with_context(|| context!());
            }
        })
    }
    pub fn local_var_from(
        operand: &llvm_ir::Operand,
        regs: &HashMap<Name, Reg>,
    ) -> Result<Operand> {
        Ok(match operand {
            llvm_ir::Operand::LocalOperand { name, ty: _ } => {
                let reg = regs
                    .get(name)
                    .ok_or(anyhow!("local var not found {}", name))
                    .with_context(|| context!())?;
                reg.into()
            }
            _ => {
                return Err(anyhow!("operand is not local var:{}", operand))
                    .with_context(|| context!());
            }
        })
    }
    pub fn label_name_from(name: &llvm_ir::Name) -> Result<String> {
        let name = name.to_string();
        let name = &name
            .strip_prefix('%')
            .ok_or(anyhow!("").context(context!()))?;
        Ok(name.to_string())
    }

    pub fn const_from(operand: &llvm_ir::Operand) -> Result<Operand> {
        Ok(match operand {
            llvm_ir::Operand::ConstantOperand(c) => match c.as_ref() {
                Constant::Int { bits: _bits, value } => Operand::Imm((*value as i64).into()),
                Constant::Float(f) => match f {
                    llvm_ir::constant::Float::Single(f) => Operand::Fmm((*f as f64).into()),
                    llvm_ir::constant::Float::Double(_) => {
                        unimplemented!("double float");
                    }
                    _ => {
                        unreachable!();
                    }
                },
                _ => todo!(),
            },
            _ => {
                return Err(anyhow!("operand is not constant:{}", operand))
                    .with_context(|| context!())
            }
        })
    }

    pub fn value_from(operand: &llvm_ir::Operand, regs: &HashMap<Name, Reg>) -> Result<Operand> {
        if let Ok(c) = Self::const_from(operand) {
            Ok(c)
        } else if let Ok(c) = Self::local_var_from(operand, regs) {
            Ok(c)
        } else {
            Err(anyhow!("value neither is reg or const:{}", operand)).with_context(|| context!())
        }
    }

    #[inline]
    pub fn global_name_from(operand: &llvm_ir::Operand) -> Result<String> {
        let n = match operand {
            llvm_ir::Operand::LocalOperand { name: _, ty: _ } => {
                return Err(anyhow!("local operand".to_string())).with_context(|| context!())
            }
            llvm_ir::Operand::ConstantOperand(c) => match c.as_ref() {
                Constant::GlobalReference { name, ty: _ } => name.clone(),
                _ => todo!(),
            },
            llvm_ir::Operand::MetadataOperand => todo!(),
        };
        Self::name_without_prefix(&n)
    }

    #[inline]
    pub fn func_name_from(operand: &llvm_ir::Operand) -> Result<String> {
        let name = match operand {
            llvm_ir::Operand::LocalOperand { name: _, ty: _ } => {
                Err(anyhow!("local operand".to_string())).with_context(|| context!())
            }
            llvm_ir::Operand::ConstantOperand(c) => match c.as_ref() {
                Constant::GlobalReference { name, ty: _ } => Ok(name.clone()),
                _ => todo!(),
            },
            llvm_ir::Operand::MetadataOperand => todo!(),
        }?;
        Self::name_without_prefix(&name)
    }

    #[inline]
    pub fn name_without_prefix(name: &llvm_ir::Name) -> Result<String> {
        let name = name.to_string();
        name.strip_prefix('%')
            .ok_or(anyhow!("").context(context!()))
            .map(|s| s.to_string())
    }
}
