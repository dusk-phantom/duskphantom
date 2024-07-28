use std::collections::HashMap;

use super::*;

use builder::IRBuilder;

use llvm_ir::{Constant, Name};
use var::FloatVar;

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
                _ => todo!(),
            },
            _ => {
                return Err(anyhow!("operand is not constant:{}", operand))
                    .with_context(|| context!())
            }
        })
    }

    /// if operand is Imm imm ,return Operand::Imm(imm)
    /// if operand is Reg reg ,return Operand::Reg(reg)
    /// if operand is Fmm fmm ,return Operand::Fmm(fmm)
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
    /// so this function promise that the return value is a (reg,pre_insts) tuple
    /// pre_insts is the insts that generate the reg,which should be inserted before the insts that use the reg
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
    ) -> Result<(Operand, Vec<Inst>)> {
        let mut pre_insert = Vec::new();
        let addr: Operand = match operand {
            llvm_ir::Operand::LocalOperand { name: _, ty: _ } => {
                Self::stack_slot_from(operand, stack_slots)?
            }
            llvm_ir::Operand::ConstantOperand(c) => match c.as_ref() {
                Constant::GlobalReference { name: _, ty: _ } => {
                    Self::globalname_from_operand(operand).map(|s| s.into())?
                }
                Constant::GetElementPtr(gep) => {
                    dbg!(gep);
                    match gep.address.as_ref() {
                        Constant::GlobalReference { name: _, ty } => {
                            let arr_label = Self::globalname_from_operand(operand)?;
                            // dbg!(gep);
                            // lla base <arr_label>; mul of0 <index[0]> <size[0]>; add base base of0
                            let dims = Self::dimensions_from_array(ty)?;
                            let idxs = Self::indices_from_gep(gep)?;
                            assert!(idxs.len() == dims.len());
                            let base_reg = reg_gener.gen_virtual_usual_reg();
                            let lla = LlaInst::new(base_reg, arr_label.into());
                            pre_insert.push(lla.into());

                            let mut sizes = {
                                let mut sizes = Vec::new();
                                let mut size = 1;
                                for dim in dims.iter().rev() {
                                    sizes.push(size);
                                    size *= dim;
                                }
                                sizes.reverse();
                                sizes
                            };
                            assert!(sizes.len() == idxs.len());

                            let mut offset = REG_ZERO;
                            for (idx, factor) in idxs.iter().zip(sizes.iter()) {
                                if factor == &1 {
                                    let to_add: Imm = (*idx as i64).into();
                                    let rhs: Operand = if to_add.in_limit(12) {
                                        to_add.clone().into()
                                    } else {
                                        let rhs = reg_gener.gen_virtual_usual_reg();
                                        let li = LiInst::new(rhs.into(), to_add.clone().into());
                                        pre_insert.push(li.into());
                                        rhs.into()
                                    };

                                    let new_offset = reg_gener.gen_virtual_usual_reg();
                                    let add = AddInst::new(
                                        new_offset.into(),
                                        offset.into(),
                                        to_add.into(),
                                    );
                                    offset = new_offset;
                                    pre_insert.push(add.into());
                                    continue;
                                }

                                // Note!!! the factor is a usize, so such conversion is not always safe
                                let factor: Imm = (*factor as i64).into();
                                let to_add = reg_gener.gen_virtual_usual_reg();
                                let lhs = reg_gener.gen_virtual_usual_reg();

                                // Note!!! the idx is a usize, so it is not always save to convert it to i64
                                let li = LiInst::new(lhs.into(), (*idx as i64).into());
                                pre_insert.push(li.into());

                                let rhs = if factor.in_limit(12) {
                                    factor.into()
                                } else {
                                    let rhs = reg_gener.gen_virtual_usual_reg();
                                    let li = LiInst::new(rhs.into(), factor.into());
                                    pre_insert.push(li.into());
                                    rhs.into()
                                };

                                let mul = MulInst::new(to_add.into(), lhs.into(), rhs);
                                pre_insert.push(mul.into());

                                let new_offset = reg_gener.gen_virtual_usual_reg();
                                let add =
                                    AddInst::new(new_offset.into(), offset.into(), to_add.into());
                                offset = new_offset;
                                pre_insert.push(add.into());
                            }

                            let addr = reg_gener.gen_virtual_usual_reg();
                            let add = AddInst::new(addr.into(), base_reg.into(), offset.into());
                            pre_insert.push(add.into());
                            addr.into()
                        }
                        _ => unimplemented!(),
                    }
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
    pub fn dimensions_from_array(arr_ty: &llvm_ir::Type) -> Result<Vec<usize>> {
        match arr_ty {
            llvm_ir::Type::ArrayType {
                element_type,
                num_elements,
            } => {
                let mut dims = vec![*num_elements];
                let suf_dims = Self::dimensions_from_array(element_type)?;
                dims.extend(suf_dims);
                Ok(dims)
            }
            llvm_ir::Type::IntegerType { bits: _ } => Ok(vec![]),
            llvm_ir::Type::FPType(_) => Ok(vec![]),
            _ => unimplemented!(),
        }
    }

    #[inline]
    pub fn indices_from_gep(gep: &llvm_ir::constant::GetElementPtr) -> Result<Vec<usize>> {
        let mut indices = Vec::new();
        fn idx_from(c: &Constant) -> Result<usize> {
            match c {
                Constant::Int { bits: _, value } => Ok(*value as usize),
                _ => {
                    dbg!(c);
                    unimplemented!();
                }
            }
        }
        for idx in &gep.indices {
            indices.push(idx_from(idx)?);
        }
        Ok(indices)
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
