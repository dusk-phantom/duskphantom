use builder::IRBuilder;

use super::*;
use common::Dimension;
use llvm_ir::{Constant, Name};
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
        let mut offset = reg_gener.gen_virtual_usual_reg();

        // process first index
        let Some(first_idx) = gep.indices.first() else {
            unreachable!();
        };
        let (new_offset, insts) =
            Self::process_first_idx(offset, first_idx, &dims, reg_gener, regs)?;
        ret.extend(insts);
        offset = new_offset;

        let mut dims = Some(&dims);
        for idx in gep.indices.iter() {
            if let Some(d) = dims {
                let (new_offset, new_dims, insts) =
                    Self::process_sub_idx(offset, idx, d, reg_gener, regs)?;
                ret.extend(insts);
                offset = new_offset;
                dims = new_dims;
            } else {
                break;
            }
        }

        let final_offset = reg_gener.gen_virtual_usual_reg();
        let slli = SllInst::new(final_offset.into(), offset.into(), 2.into());
        ret.push(slli.into());

        let final_addr = reg_gener.gen_virtual_usual_reg();
        let add = AddInst::new(final_addr.into(), base.into(), final_offset.into());
        ret.push(add.into());

        regs.insert(gep.dest.clone(), final_addr);

        Ok(ret)
    }

    fn process_first_idx(
        offset: Reg,
        idx: &llvm_ir::Operand,
        dims: &Dimension,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<(Reg, Vec<Inst>)> {
        let mut ret_insts: Vec<Inst> = Vec::new();
        let mut offset = offset;

        let idx = Self::value_from(idx, regs)?;

        let factor: Imm = dims.size().try_into()?;
        let (factor, pre_insert) = Self::prepare_imm_rhs(&factor, reg_gener)?;
        ret_insts.extend(pre_insert);

        match idx {
            Operand::Imm(idx_imm) => {
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

                offset = new_offset;
            }
            Operand::Reg(idx) => {
                let to_add = reg_gener.gen_virtual_usual_reg();
                let new_offset = reg_gener.gen_virtual_usual_reg();

                let mul: Inst = MulInst::new(to_add.into(), idx.into(), factor).into();
                ret_insts.push(mul);

                let add: Inst =
                    AddInst::new(new_offset.into(), offset.into(), to_add.into()).into();
                ret_insts.push(add);

                offset = new_offset;
            }
            _ => unimplemented!(),
        }

        Ok((offset, ret_insts))
    }

    #[allow(unused)]
    fn process_sub_idx<'a>(
        offset: Reg,
        idx: &llvm_ir::Operand,
        dims: &'a Dimension,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<(Reg, Option<&'a Dimension>, Vec<Inst>)> {
        let mut ret_insts: Vec<Inst> = Vec::new();
        let mut offset = offset;
        let mut ret_dims = None;

        let idx = Self::value_from(idx, regs)?;
        match idx {
            Operand::Imm(idx_imm) => {
                // 根据idx_imm的值,计算offset的增量
                let idx_u: usize = idx_imm.try_into()?;
                let factor: usize = dims.iter_subs().take(idx_u).map(|d| d.size()).sum();
                let factor: Imm = factor.try_into()?;
                let new_offset = reg_gener.gen_virtual_usual_reg();
                let add = AddInst::new(new_offset.into(), offset.into(), factor.into());
                ret_insts.push(add.into());
                offset = new_offset;
                ret_dims = dims.iter_subs().nth(idx_u);
            }
            Operand::Reg(idx) => {
                unimplemented!();
            }
            _ => {
                return Err(anyhow!("").context(context!()));
            }
        }

        Ok((offset, ret_dims, ret_insts))
    }

    fn dims_from_gep_inst(gep: &llvm_ir::instruction::GetElementPtr) -> Result<Dimension> {
        match &gep.address {
            llvm_ir::Operand::LocalOperand { name: _, ty } => Self::dims_from_ty(ty),
            llvm_ir::Operand::ConstantOperand(c) => match c.as_ref() {
                Constant::GlobalReference { name: _, ty } => Self::dims_from_ty(ty),
                _ => unimplemented!(),
            },
            llvm_ir::Operand::MetadataOperand => Err(anyhow!("").context(context!())),
        }
    }
}
