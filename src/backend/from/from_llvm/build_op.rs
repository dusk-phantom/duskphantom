use std::collections::HashMap;

use super::*;

use llvm_ir::{Constant, Name};
// use var::FloatVar;

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

    pub fn is_ty_ptr(ty: &llvm_ir::Type) -> bool {
        matches!(ty, llvm_ir::Type::PointerType { addr_space: _ })
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

    pub fn reg_from(operand: &llvm_ir::Operand, regs: &HashMap<Name, Reg>) -> Result<Operand> {
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

    pub fn local_var_from(
        operand: &llvm_ir::Operand,
        stack_slots: &HashMap<Name, StackSlot>,
        regs: &HashMap<Name, Reg>,
    ) -> Result<Operand> {
        Ok(match operand {
            llvm_ir::Operand::LocalOperand { name, ty: _ } => {
                if let Some(ss) = stack_slots.get(name) {
                    ss.into()
                } else {
                    let reg = regs
                        .get(name)
                        .ok_or(anyhow!("local var not found {}", name))
                        .with_context(|| context!())?;
                    reg.into()
                }
            }
            _ => {
                return Err(anyhow!("operand is not local var:{}", operand))
                    .with_context(|| context!());
            }
        })
    }

    pub fn stack_slot_from(
        operand: &llvm_ir::Operand,
        stack_slots: &HashMap<Name, StackSlot>,
    ) -> Result<StackSlot> {
        Ok(match operand {
            llvm_ir::Operand::LocalOperand { name, ty: _ } => stack_slots
                .get(name)
                .cloned()
                .ok_or(anyhow!("stack slot not found {}", name))?,
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
        Ok(format!(".LBB_{}", name))
    }

    pub fn const_from(operand: &llvm_ir::Operand) -> Result<Operand> {
        Ok(match operand {
            llvm_ir::Operand::ConstantOperand(c) => match c.as_ref() {
                Constant::Int { bits: _bits, value } => Operand::Imm((*value as i64).into()),
                Constant::Float(f) => match f {
                    llvm_ir::constant::Float::Single(f) => Operand::Fmm((*f as f64).into()),
                    llvm_ir::constant::Float::Double(d) => Operand::Fmm(d.into()),
                    _ => {
                        unreachable!();
                    }
                },
                Constant::GlobalReference { name: _, ty: _ } => {
                    // unimplemented!();
                    let name = Self::globalname_from_constant(c)?;
                    name.into()
                }
                _ => {
                    dbg!(c);
                    unimplemented!();
                }
            },
            _ => {
                return Err(anyhow!("operand is not constant:{}", operand))
                    .with_context(|| context!())
            }
        })
    }

    /// value must be a fmm,imm, or a reg
    pub fn value_from(operand: &llvm_ir::Operand, regs: &HashMap<Name, Reg>) -> Result<Operand> {
        if let Ok(c) = Self::const_from(operand) {
            Ok(c)
        } else if let Ok(c) = Self::reg_from(operand, regs) {
            Ok(c)
        } else {
            Err(anyhow!("value neither is reg or const:{}", operand)).with_context(|| context!())
        }
    }

    pub fn prepare_imm_rhs(
        imm: &Imm,
        reg_gener: &mut RegGenerator,
    ) -> Result<(Operand, Option<Inst>)> {
        if imm.in_limit(12) {
            Ok((Operand::Imm(*imm), None))
        } else {
            let dst = reg_gener.gen_virtual_usual_reg();
            let li = LiInst::new(dst.into(), imm.into());
            Ok((dst.into(), Some(li.into())))
        }
    }

    #[inline]
    pub fn prepare_imm_lhs(
        imm: &Imm,
        reg_gener: &mut RegGenerator,
    ) -> Result<(Operand, Option<Inst>)> {
        let dst = reg_gener.gen_virtual_usual_reg();
        let li = LiInst::new(dst.into(), imm.into());
        Ok((dst.into(), Some(li.into())))
    }

    /// 需要注意的是 指令的 lvalue 只能是寄存器,所以如果value是个常数,则需要用一个寄存器来存储,并且需要生成一条指令
    /// so this function promise that the return value is a (reg,pre_insts) tuple
    /// pre_insts is the insts that generate the reg,which should be inserted before the insts that use the reg
    #[allow(unused)]
    pub fn prepare_lhs(
        value: &llvm_ir::operand::Operand,
        reg_gener: &mut RegGenerator,
        regs: &HashMap<Name, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
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
            Operand::Fmm(fmm) => {
                let f_var = Self::fmm_from(fmm, fmms)?;
                let addr = reg_gener.gen_virtual_usual_reg();
                let lla = LlaInst::new(addr, f_var.name.clone().into());
                insts.push(lla.into());
                let fmm = reg_gener.gen_virtual_float_reg();
                let lf = LwInst::new(fmm, 0.into(), addr);
                insts.push(lf.into());
                Ok((fmm.into(), insts))
            }
            _ => {
                dbg!(value);
                unimplemented!();
            }
        }
    }

    // this function is used to prepare the lhs of a usual operand
    #[allow(unused)]
    pub fn prepare_usual_lhs(
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
            _ => Err(anyhow!("value is not usual operand:{:?}", value)).with_context(|| context!()),
        }
    }

    #[allow(unused)]
    pub fn prepare_float_lhs(
        value: &llvm_ir::operand::Operand,
        reg_gener: &mut RegGenerator,
        regs: &HashMap<Name, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
    ) -> Result<(Operand, Vec<Inst>)> {
        let mut insts = Vec::new();
        let value = IRBuilder::value_from(value, regs)?;
        match &value {
            Operand::Reg(_) => Ok((value, insts)),
            Operand::Fmm(fmm) => {
                let f_var = Self::fmm_from(fmm, fmms)?;
                let addr = reg_gener.gen_virtual_usual_reg();
                let lla = LlaInst::new(addr, f_var.name.clone().into());
                insts.push(lla.into());
                let fmm = reg_gener.gen_virtual_float_reg();
                let lf = LwInst::new(fmm, 0.into(), addr);
                insts.push(lf.into());
                Ok((fmm.into(), insts))
            }
            _ => Err(anyhow!("value is not float operand:{:?}", value)).with_context(|| context!()),
        }
    }

    /// 如果value是个寄存器,直接返回,
    /// 如果是个常数,如果超出范围,则需要用一个寄存器来存储,并且需要生成一条指令
    /// 如果是不超出范围的常数,则直接返回
    /// this function is used to prepare the rhs of a usual or float operand
    pub fn prepare_rhs(
        value: &llvm_ir::operand::Operand,
        reg_gener: &mut RegGenerator,
        regs: &HashMap<Name, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
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
            Operand::Fmm(fmm) => {
                let f_var = Self::fmm_from(fmm, fmms)?;
                let addr = reg_gener.gen_virtual_usual_reg();
                let lla = LlaInst::new(addr, f_var.name.clone().into());
                insts.push(lla.into());
                let fmm = reg_gener.gen_virtual_float_reg();
                let lf = LwInst::new(fmm, 0.into(), addr);
                insts.push(lf.into());
                Ok((fmm.into(), insts))
            }
            _ => unimplemented!(),
        }
    }

    /// this function is used to prepare the rhs of a usual operand
    pub fn prepare_usual_rhs(
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

    /// this function is used to prepare the rhs of a float operand
    pub fn prepare_float_rhs(
        value: &llvm_ir::operand::Operand,
        reg_gener: &mut RegGenerator,
        regs: &HashMap<Name, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
    ) -> Result<(Operand, Vec<Inst>)> {
        let mut insts: Vec<Inst> = Vec::new();
        let value = IRBuilder::value_from(value, regs)?;
        match &value {
            Operand::Reg(_) => Ok((value, insts)),
            Operand::Fmm(fmm) => {
                let f_var = Self::fmm_from(fmm, fmms)?;
                let addr = reg_gener.gen_virtual_usual_reg();
                let lla = LlaInst::new(addr, f_var.name.clone().into());
                insts.push(lla.into());
                let fmm = reg_gener.gen_virtual_float_reg();
                let lf = LwInst::new(fmm, 0.into(), addr);
                insts.push(lf.into());
                Ok((fmm.into(), insts))
            }
            _ => unimplemented!(),
        }
    }

    #[inline]
    #[allow(unused)]
    pub fn prepare_address(
        operand: &llvm_ir::Operand,
        reg_gener: &mut RegGenerator,
        stack_slots: &HashMap<Name, StackSlot>,
        regs: &HashMap<Name, Reg>,
    ) -> Result<(Operand, Vec<Inst>)> {
        let mut pre_insert = Vec::new();
        let addr: Operand = match operand {
            llvm_ir::Operand::LocalOperand { name, ty: _ } => {
                Self::local_var_from(operand, stack_slots, regs)?
            }
            llvm_ir::Operand::ConstantOperand(c) => match c.as_ref() {
                Constant::GlobalReference { name: _, ty: _ } => {
                    Self::globalname_from_operand(operand).map(|s| s.into())?
                }
                Constant::GetElementPtr(gep) => {
                    return Self::prepare_gep_address(gep, reg_gener, stack_slots, regs);
                }
                _ => {
                    dbg!(c);
                    unimplemented!();
                }
            },
            _ => {
                return Err(anyhow!("operand is not local var:{}", operand))
                    .with_context(|| context!())
            }
        };
        Ok((addr, pre_insert))
    }

    #[allow(unused)]
    #[allow(clippy::diverging_sub_expression)]
    fn prepare_gep_address(
        gep: &llvm_ir::constant::GetElementPtr,
        _reg_gener: &mut RegGenerator,
        _stack_slots: &HashMap<Name, StackSlot>,
        _regs: &HashMap<Name, Reg>,
    ) -> Result<(Operand, Vec<Inst>)> {
        let mut pre_insert: Vec<Inst> = Vec::new();

        let base: Operand = match gep.address.as_ref() {
            Constant::AggregateZero(_) => todo!(),
            Constant::Struct {
                name,
                values,
                is_packed,
            } => todo!(),
            Constant::Array {
                element_type,
                elements,
            } => todo!(),
            Constant::GlobalReference { name, ty } => todo!(),
            Constant::GetElementPtr(gep) => {
                let (addr, sub_pre_insert) =
                    Self::prepare_gep_address(gep, _reg_gener, _stack_slots, _regs)?;
                pre_insert.extend(sub_pre_insert);
                addr
            }
            _ => todo!(),
        };
        let final_addr = todo!();
        unimplemented!();
        Ok((final_addr, pre_insert))
    }

    #[inline]
    pub fn fmm_from_constant<'a>(
        constant: &llvm_ir::constant::Constant,
        fmms: &'a mut HashMap<Fmm, FloatVar>,
    ) -> Result<&'a FloatVar> {
        match constant {
            Constant::Float(f) => match f {
                llvm_ir::constant::Float::Single(f) => {
                    let fmm: Fmm = f.into();
                    Self::fmm_from(&fmm, fmms)
                }
                llvm_ir::constant::Float::Double(_) => {
                    unimplemented!("double float");
                }
                _ => {
                    unreachable!();
                }
            },
            _ => todo!(),
        }
    }

    #[inline]
    pub fn fmm_from<'a>(fmm: &Fmm, fmms: &'a mut HashMap<Fmm, FloatVar>) -> Result<&'a FloatVar> {
        let init = fmm.try_into()?;
        let new_f_var = || -> FloatVar {
            let name = format!("_fc_{:X}", fmm.to_bits());
            FloatVar {
                name: name.clone(),
                init: Some(init),
                is_const: true,
            }
        };
        fmms.entry(fmm.clone()).or_insert_with(new_f_var);
        fmms.get(fmm).ok_or(anyhow!("")).with_context(|| context!())
    }

    #[inline]
    pub fn mem_size_from(ty: &llvm_ir::Type) -> Result<MemSize> {
        match ty {
            llvm_ir::Type::IntegerType { bits: 8 } => unimplemented!(),
            llvm_ir::Type::IntegerType { bits: 16 } => unimplemented!(),
            llvm_ir::Type::IntegerType { bits: 32 } => Ok(MemSize::FourByte),
            llvm_ir::Type::IntegerType { bits: 64 } => Ok(MemSize::EightByte),
            llvm_ir::Type::FPType(f) => match f {
                llvm_ir::types::FPType::Half => todo!(),
                llvm_ir::types::FPType::BFloat => todo!(),
                llvm_ir::types::FPType::Single => Ok(MemSize::FourByte),
                llvm_ir::types::FPType::Double => Ok(MemSize::EightByte),
                llvm_ir::types::FPType::FP128 => todo!(),
                llvm_ir::types::FPType::X86_FP80 => todo!(),
                llvm_ir::types::FPType::PPC_FP128 => todo!(),
            },
            _ => Err(anyhow!("mem size not supported").context(context!())),
        }
    }

    #[inline]
    pub fn globalname_from_operand(operand: &llvm_ir::Operand) -> Result<String> {
        match operand {
            llvm_ir::Operand::LocalOperand { name: _, ty: _ } => {
                Err(anyhow!("local operand".to_string())).with_context(|| context!())
            }
            llvm_ir::Operand::ConstantOperand(c) => {
                Self::globalname_from_constant(c).with_context(|| context!())
            }
            llvm_ir::Operand::MetadataOperand => todo!(),
        }
    }

    fn globalname_from_constant(c: &Constant) -> Result<String> {
        let n = match c {
            Constant::GlobalReference { name, ty: _ } => name.clone(),
            Constant::GetElementPtr(gep) => match gep.address.as_ref() {
                Constant::GlobalReference { name, ty: _ } => name.clone(),
                _ => unimplemented!(),
            },
            _ => {
                dbg!(c);
                unimplemented!();
            }
        };
        IRBuilder::name_without_prefix(&n)
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
