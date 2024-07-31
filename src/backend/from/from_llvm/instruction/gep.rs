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
        dbg!(gep.to_string());
        dbg!(&dims);

        // case 1: valid indices could last with only one local var idx, if there is a var idx, it must be the first one
        // case 2: valid indices could only contains imm.
        // FIXME: update dims to sub item
        let mut first_var_idx_occur = false;
        if let Some(idx) = gep.indices.first() {
            let idx = Self::value_from(idx, regs)?;

            let factor: Imm = dims.size().try_into()?;
            let (factor, pre_insert) = Self::prepare_imm_rhs(&factor, reg_gener)?;
            ret.extend(pre_insert);

            match idx {
                Operand::Imm(imm) => {
                    let to_add = reg_gener.gen_virtual_usual_reg();
                    let idx = reg_gener.gen_virtual_usual_reg();
                    let new_offset = reg_gener.gen_virtual_usual_reg();

                    let li: Inst = LiInst::new(idx.into(), imm.into()).into();
                    ret.push(li);

                    let mul: Inst = MulInst::new(to_add.into(), idx.into(), factor).into();
                    ret.push(mul);

                    let add: Inst =
                        AddInst::new(new_offset.into(), offset.into(), to_add.into()).into();
                    ret.push(add);

                    offset = new_offset;

                    // update dims
                }
                Operand::Reg(reg) => {}
                _ => unimplemented!(),
            }
        }
        for idx in gep.indices.iter().skip(1) {
            todo!("gep with multiple indices");
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
