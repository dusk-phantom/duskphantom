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

    pub fn address_from(
        operand: &llvm_ir::Operand,
        stack_slots: &HashMap<Name, StackSlot>,
    ) -> Result<Operand> {
        Ok(match operand {
            llvm_ir::Operand::LocalOperand { name, ty: _ } => stack_slots
                .get(name)
                .ok_or(anyhow!("stack slot not found {}", name))
                .with_context(|| context!())?
                .into(),
            llvm_ir::Operand::ConstantOperand(_) => todo!(),
            llvm_ir::Operand::MetadataOperand => todo!(),
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

    #[allow(unused)]
    #[inline]
    pub fn global_name_from(operand: &llvm_ir::Operand) -> Result<Name> {
        match operand {
            llvm_ir::Operand::LocalOperand { name: _, ty: _ } => {
                Err(anyhow!("local operand".to_string())).with_context(|| context!())
            }
            llvm_ir::Operand::ConstantOperand(c) => match c.as_ref() {
                Constant::GlobalReference { name, ty: _ } => Ok(name.clone()),
                _ => todo!(),
            },
            llvm_ir::Operand::MetadataOperand => todo!(),
        }
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
        let f_name = name.to_string();
        let f_name = &f_name
            .strip_prefix('%')
            .ok_or(anyhow!("").context(context!()))?;
        Ok(f_name.to_string())
    }
}
