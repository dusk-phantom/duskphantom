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

    /// 需要注意的是 指令的 lvalue 只能是寄存器,所以如果value是个常数,则需要用一个寄存器来存储,并且需要生成一条指令
    #[allow(unused)]
    pub fn prepare_lhs(
        value: &llvm_ir::operand::Operand,
        reg_gener: &mut RegGenerator,
        regs: &HashMap<Name, Reg>,
    ) -> Result<(Operand, Vec<Inst>)> {
        let mut insts = Vec::new();
        let value = IRBuilder::value_from(value, regs)?;
        match &value {
            Operand::Imm(imm) => {
                let dst = reg_gener.gen_virtual_usual_reg();
                let li = LiInst::new(dst.into(), imm.into());
                insts.push(li.into());
                Ok((dst.into(), insts))
            }
            Operand::Reg(_) => Ok((value, insts)),
            _ => unimplemented!(),
        }
    }

    /// 如果value是个寄存器,直接返回,
    /// 如果是个常数,如果超出范围,则需要用一个寄存器来存储,并且需要生成一条指令
    /// 如果是不超出范围的常数,则直接返回
    pub fn prepare_rhs(
        value: &llvm_ir::operand::Operand,
        reg_gener: &mut RegGenerator,
        regs: &HashMap<Name, Reg>,
    ) -> Result<(Operand, Vec<Inst>)> {
        let mut insts: Vec<Inst> = Vec::new();
        let value = IRBuilder::value_from(value, regs)?;
        match &value {
            Operand::Imm(imm) => {
                if imm.in_limit(12) {
                    Ok((value, insts))
                } else {
                    let dst = reg_gener.gen_virtual_usual_reg();
                    let li = LiInst::new(dst.into(), imm.into());
                    insts.push(li.into());
                    Ok((dst.into(), insts))
                }
            }
            Operand::Reg(_) => Ok((value, insts)),
            _ => unimplemented!(),
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
