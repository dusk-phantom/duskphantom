use builder::IRBuilder;

use super::*;
use common::Dimension;
use llvm_ir::Name;
use std::collections::HashMap;

impl IRBuilder {
    #[allow(unreachable_code)]
    #[allow(unused)]
    pub fn build_gep_inst(
        gep: &llvm_ir::instruction::GetElementPtr,
        stack_slots: &mut HashMap<Name, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        #[inline]
        fn prepare_rhs_op(
            op: &Operand,
            reg_gener: &mut RegGenerator,
        ) -> Result<(Operand, Vec<Inst>)> {
            match &op {
                Operand::Imm(imm) => {
                    if imm.in_limit(12) {
                        Ok((op.clone(), vec![]))
                    } else {
                        let r = reg_gener.gen_virtual_usual_reg();
                        let li = LiInst::new(r.into(), op.clone());
                        Ok((r.into(), vec![li.into()]))
                    }
                }
                Operand::Reg(reg) => Ok((op.clone(), vec![])),
                _ => unreachable!(),
            }
        }
        #[inline]
        fn prepare_lhs_reg(
            op: &Operand,
            reg_gener: &mut RegGenerator,
        ) -> Result<(Operand, Vec<Inst>)> {
            match op {
                Operand::Imm(imm) => {
                    let r = reg_gener.gen_virtual_usual_reg();
                    let li = LiInst::new(r.into(), imm.into());
                    Ok((r.into(), vec![li.into()]))
                }
                Operand::Reg(reg) => Ok((op.clone(), vec![])),
                _ => unreachable!(),
            }
        }

        dbg!(gep);
        let mut ret: Vec<Inst> = Vec::new();
        let (addr, pre_insert) = Self::prepare_address(&gep.address, reg_gener, stack_slots, regs)
            .with_context(|| context!())?;
        ret.extend(pre_insert);

        // prepare base address
        let base = match addr {
            Operand::StackSlot(stack_slot) => {
                let addr = reg_gener.gen_virtual_usual_reg();
                let laddr = LocalAddr::new(addr, stack_slot);
                ret.push(laddr.into());
                dbg!(stack_slot);
                todo!();
                addr
            }
            Operand::Label(var) => {
                let addr = reg_gener.gen_virtual_usual_reg();
                let lla = LlaInst::new(addr, var);
                ret.push(lla.into());
                addr
            }
            Operand::Reg(reg) => reg,
            _ => {
                dbg!(addr);
                unimplemented!();
            }
        };
        let dims = Self::dims_from_gep_inst(gep)?;

        // process first index
        let Some(first_idx) = gep.indices.first() else {
            unreachable!();
        };
        let (offset, insts) = Self::process_first_idx(first_idx, &dims, reg_gener, regs)?;
        ret.extend(insts);

        let mut offset = offset.map(Operand::from);

        let mut dims = Some(&dims);
        for idx in gep.indices.iter().skip(1) {
            if let Some(d) = dims {
                let (to_add, new_dims, insts) = Self::process_sub_idx(idx, d, reg_gener, regs)?;
                ret.extend(insts);

                if let Some(off) = offset {
                    let new_offset = reg_gener.gen_virtual_usual_reg();

                    let (to_add, pre_insert) = prepare_rhs_op(&to_add, reg_gener)?;
                    ret.extend(pre_insert);

                    let (off, pre_insert) = prepare_lhs_reg(&off, reg_gener)?;
                    ret.extend(pre_insert);

                    let add = AddInst::new(new_offset.into(), off, to_add);
                    ret.push(add.into());

                    offset = Some(new_offset.into());
                } else {
                    offset = Some(to_add);
                }

                dims = new_dims;
            } else {
                break;
            }
        }

        let final_offset = reg_gener.gen_virtual_usual_reg();
        let (offset, pre_insert) = prepare_lhs_reg(&offset.unwrap(), reg_gener)?;
        ret.extend(pre_insert);

        let slli = SllInst::new(final_offset.into(), offset, 2.into());
        ret.push(slli.into());

        let final_addr = reg_gener.gen_virtual_usual_reg();
        let add = AddInst::new(final_addr.into(), base.into(), final_offset.into());
        ret.push(add.into());

        regs.insert(gep.dest.clone(), final_addr);

        println!(
            "{}",
            &ret.iter()
                .map(|i| i.gen_asm())
                .collect::<Vec<_>>()
                .join("\n")
        );
        Ok(ret)
    }

    fn process_first_idx(
        idx: &llvm_ir::Operand,
        dims: &Dimension,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<(Option<Reg>, Vec<Inst>)> {
        let mut ret_insts: Vec<Inst> = Vec::new();

        let idx = Self::value_from(idx, regs)?;

        let mut get_factor = |dims: &Dimension| -> Result<Operand> {
            let factor: Imm = dims.size().try_into()?;
            let (factor, pre_insert) = Self::prepare_imm_lhs(&factor, reg_gener)?;
            ret_insts.extend(pre_insert);
            Ok(factor)
        };

        let ret_offset = match idx {
            Operand::Imm(idx_imm) => {
                if idx_imm == 0.into() {
                    return Ok((None, ret_insts));
                }
                let factor = get_factor(dims)?;
                let offset = reg_gener.gen_virtual_usual_reg();

                let to_add = reg_gener.gen_virtual_usual_reg();
                let idx_reg = reg_gener.gen_virtual_usual_reg();
                let new_offset = reg_gener.gen_virtual_usual_reg();

                let li: Inst = LiInst::new(idx_reg.into(), idx_imm.into()).into();
                ret_insts.push(li);

                let mul: Inst = MulInst::new(to_add.into(), idx_reg.into(), factor).into();
                ret_insts.push(mul);

                let add: Inst =
                    AddInst::new(new_offset.into(), offset.into(), to_add.into()).into();
                ret_insts.push(add);

                Some(new_offset)
            }
            Operand::Reg(idx) => {
                let factor = get_factor(dims)?;
                let offset = reg_gener.gen_virtual_usual_reg();
                let to_add = reg_gener.gen_virtual_usual_reg();
                let new_offset = reg_gener.gen_virtual_usual_reg();

                let mul: Inst = MulInst::new(to_add.into(), idx.into(), factor).into();
                ret_insts.push(mul);

                let add: Inst =
                    AddInst::new(new_offset.into(), offset.into(), to_add.into()).into();
                ret_insts.push(add);

                Some(new_offset)
            }
            _ => unimplemented!(),
        };

        Ok((ret_offset, ret_insts))
    }

    #[allow(unused)]
    /// return (to_add, new_dims, insts)
    fn process_sub_idx<'a>(
        idx: &llvm_ir::Operand,
        dims: &'a Dimension,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<(Operand, Option<&'a Dimension>, Vec<Inst>)> {
        let mut ret_insts: Vec<Inst> = Vec::new();
        let mut ret_dims = None;

        let idx = Self::value_from(idx, regs)?;
        let to_add: Operand = match idx {
            Operand::Imm(idx_imm) => {
                // 根据idx_imm的值,计算offset的增量
                let idx_u: usize = idx_imm.try_into()?;
                let factor: usize = dims.iter_subs().take(idx_u).map(|d| d.size()).sum();
                let factor: Imm = factor.try_into()?;
                ret_dims = dims.iter_subs().nth(idx_u);
                factor.into()
            }
            Operand::Reg(idx) => {
                dbg!(dims);
                assert!(dims.is_array_like());
                let e_dim = dims.iter_subs().next();
                let factor = if let Some(e_dim) = e_dim {
                    let factor: Imm = e_dim.size().try_into()?;
                    factor
                } else {
                    1.into()
                };
                let (factor, pre_insert) = Self::prepare_imm_lhs(&factor, reg_gener)?;
                ret_insts.extend(pre_insert);

                let to_add = reg_gener.gen_virtual_usual_reg();

                let mul = MulInst::new(to_add.into(), idx.into(), factor);
                ret_insts.push(mul.into());

                ret_dims = dims.iter_subs().nth(0);
                to_add.into()
            }
            _ => {
                return Err(anyhow!("").context(context!()));
            }
        };

        Ok((to_add, ret_dims, ret_insts))
    }

    fn dims_from_gep_inst(gep: &llvm_ir::instruction::GetElementPtr) -> Result<Dimension> {
        Self::dims_from_ty(&gep.source_element_type)
    }
}
