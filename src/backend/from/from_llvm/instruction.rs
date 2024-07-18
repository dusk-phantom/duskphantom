use crate::llvm2tac_three_op_usual;

use super::*;
use builder::IRBuilder;
use llvm_ir::{Constant, Name};
use std::collections::HashMap;
use var::FloatVar;

impl IRBuilder {
    pub fn build_instruction(
        inst: &llvm_ir::Instruction,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Name, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        // dbg!(&inst);
        match inst {
            llvm_ir::Instruction::Add(add) => {
                llvm2tac_three_op_usual!(AddInst, add, reg_gener, regs)
            }
            llvm_ir::Instruction::Sub(sub) => {
                llvm2tac_three_op_usual!(SubInst, sub, reg_gener, regs)
            }
            llvm_ir::Instruction::Mul(mul) => {
                llvm2tac_three_op_usual!(MulInst, mul, reg_gener, regs)
            }
            llvm_ir::Instruction::And(and) => {
                llvm2tac_three_op_usual!(AndInst, and, reg_gener, regs)
            }
            llvm_ir::Instruction::Or(or) => llvm2tac_three_op_usual!(OrInst, or, reg_gener, regs),
            llvm_ir::Instruction::Xor(xor) => {
                llvm2tac_three_op_usual!(XorInst, xor, reg_gener, regs)
            }
            llvm_ir::Instruction::SRem(srem) => {
                llvm2tac_three_op_usual!(RemInst, srem, reg_gener, regs)
            }
            // process logical shift right
            llvm_ir::Instruction::LShr(lshr) => {
                llvm2tac_three_op_usual!(SrlInst, lshr, reg_gener, regs)
            }
            // process logical shift left
            llvm_ir::Instruction::Shl(shl) => {
                llvm2tac_three_op_usual!(SllInst, shl, reg_gener, regs)
            }
            // process arithmetic shift right
            llvm_ir::Instruction::AShr(ashr) => {
                llvm2tac_three_op_usual!(SraInst, ashr, reg_gener, regs)
            }
            llvm_ir::Instruction::UDiv(_) => todo!(),
            llvm_ir::Instruction::SDiv(_) => todo!(),
            llvm_ir::Instruction::URem(_) => todo!(),
            llvm_ir::Instruction::FAdd(_) => todo!(),
            llvm_ir::Instruction::FSub(_) => todo!(),
            llvm_ir::Instruction::FMul(_) => todo!(),
            llvm_ir::Instruction::FDiv(_) => todo!(),
            llvm_ir::Instruction::FRem(_) => todo!(),
            llvm_ir::Instruction::FNeg(_) => todo!(),
            llvm_ir::Instruction::ExtractElement(_) => todo!(),
            llvm_ir::Instruction::InsertElement(_) => todo!(),
            llvm_ir::Instruction::ShuffleVector(_) => todo!(),
            llvm_ir::Instruction::ExtractValue(_) => todo!(),
            llvm_ir::Instruction::InsertValue(_) => todo!(),
            llvm_ir::Instruction::Alloca(alloca) => {
                Self::build_alloca_inst(alloca, stack_allocator, stack_slots)
            }
            llvm_ir::Instruction::Load(load) => {
                Self::build_load_inst(load, stack_slots, reg_gener, regs)
            }
            llvm_ir::Instruction::Store(store) => {
                Self::build_store_inst(store, stack_slots, reg_gener, regs)
            }
            llvm_ir::Instruction::SIToFP(si2f) => Self::build_si2f_inst(si2f, reg_gener, regs),
            llvm_ir::Instruction::Fence(_) => todo!(),
            llvm_ir::Instruction::CmpXchg(_) => todo!(),
            llvm_ir::Instruction::AtomicRMW(_) => todo!(),
            llvm_ir::Instruction::GetElementPtr(_) => todo!(),
            llvm_ir::Instruction::Trunc(_) => todo!(),
            llvm_ir::Instruction::ZExt(_) => todo!(),
            llvm_ir::Instruction::SExt(_) => todo!(),
            llvm_ir::Instruction::FPTrunc(_) => todo!(),
            llvm_ir::Instruction::FPExt(_) => todo!(),
            llvm_ir::Instruction::FPToUI(_) => todo!(),
            llvm_ir::Instruction::FPToSI(_) => todo!(),
            llvm_ir::Instruction::UIToFP(_) => todo!(),
            llvm_ir::Instruction::PtrToInt(_) => todo!(),
            llvm_ir::Instruction::IntToPtr(_) => todo!(),
            llvm_ir::Instruction::BitCast(_) => todo!(),
            llvm_ir::Instruction::AddrSpaceCast(_) => todo!(),
            llvm_ir::Instruction::ICmp(icmp) => Self::build_icmp_inst(icmp, reg_gener, regs),
            llvm_ir::Instruction::FCmp(_) => todo!(),
            llvm_ir::Instruction::Phi(_) => todo!(),
            llvm_ir::Instruction::Select(_) => todo!(),
            llvm_ir::Instruction::Freeze(_) => todo!(),
            llvm_ir::Instruction::Call(call) => {
                Self::build_call_inst(call, stack_allocator, stack_slots, reg_gener, regs)
            }
            llvm_ir::Instruction::VAArg(_) => todo!(),
            llvm_ir::Instruction::LandingPad(_) => todo!(),
            llvm_ir::Instruction::CatchPad(_) => todo!(),
            llvm_ir::Instruction::CleanupPad(_) => todo!(),
        }
    }
    fn build_icmp_inst(
        icmp: &llvm_ir::instruction::ICmp,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        let mut ret: Vec<Inst> = Vec::new();
        match icmp.predicate {
            llvm_ir::IntPredicate::EQ => todo!(),
            llvm_ir::IntPredicate::NE => {
                let op0 = &Self::value_from(&icmp.operand0, regs)?;
                let op1 = &Self::value_from(&icmp.operand1, regs)?;
                let dest = icmp.dest.clone();
                if let (Operand::Imm(imm0), Operand::Imm(imm1)) = (op0, op1) {
                    let imm = if imm0 == imm1 { 0 } else { 1 };
                    let flag = reg_gener.gen_virtual_usual_reg();
                    let li = LiInst::new(flag.into(), imm.into());
                    regs.insert(dest, flag);
                    ret.push(li.into());
                } else if let (Operand::Reg(reg0), Operand::Reg(reg1)) = (op0, op1) {
                    assert!(reg0.is_usual() == reg1.is_usual());
                    let dst = reg_gener.gen_virtual_usual_reg();
                    let sub = SubInst::new(dst.into(), reg0.into(), reg1.into());
                    let flag = reg_gener.gen_virtual_usual_reg();
                    let seqz = SeqzInst::new(flag.into(), dst.into());
                    ret.push(sub.into());
                    ret.push(seqz.into());
                    regs.insert(dest, flag);
                } else if let (Operand::Reg(reg), Operand::Imm(imm)) = (op0, op1) {
                    let dst = reg_gener.gen_virtual_usual_reg();
                    let sub = SubInst::new(dst.into(), reg.into(), imm.into());
                    let flag = reg_gener.gen_virtual_usual_reg();
                    let seqz = SeqzInst::new(flag.into(), dst.into());
                    ret.push(sub.into());
                    ret.push(seqz.into());
                    regs.insert(dest, flag);
                } else {
                    unimplemented!();
                }
            }
            llvm_ir::IntPredicate::UGT => todo!(),
            llvm_ir::IntPredicate::UGE => todo!(),
            llvm_ir::IntPredicate::ULT => todo!(),
            llvm_ir::IntPredicate::ULE => todo!(),
            llvm_ir::IntPredicate::SGT => todo!(),
            llvm_ir::IntPredicate::SGE => todo!(),
            llvm_ir::IntPredicate::SLT => todo!(),
            llvm_ir::IntPredicate::SLE => todo!(),
        }
        Ok(ret)
    }

    fn build_si2f_inst(
        si2f: &llvm_ir::instruction::SIToFP,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        let op = Self::local_var_from(&si2f.operand, regs)?;
        let dst = Self::new_var(&si2f.to_type, reg_gener)?;
        regs.insert(si2f.dest.clone(), dst);
        let inst: Inst = if dst.is_usual() {
            F2iInst::new(dst.into(), op).into()
        } else {
            I2fInst::new(dst.into(), op).into()
        };
        Ok(vec![inst])
    }

    /// alloca instruction only instruct allocating memory on stack,not generate one-one instruction
    fn build_alloca_inst(
        alloca: &llvm_ir::instruction::Alloca,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Name, StackSlot>,
    ) -> Result<Vec<Inst>> {
        let name = alloca.dest.clone();
        let ty = alloca.allocated_type.clone();
        let num_byte = match ty.as_ref() {
            llvm_ir::Type::IntegerType { bits: _ } => 8,
            llvm_ir::Type::FPType(fp) => match fp {
                llvm_ir::types::FPType::Half => todo!(),
                llvm_ir::types::FPType::BFloat => todo!(),
                llvm_ir::types::FPType::Single => 8,
                llvm_ir::types::FPType::Double => 8,
                llvm_ir::types::FPType::FP128 => todo!(),
                llvm_ir::types::FPType::X86_FP80 => todo!(),
                llvm_ir::types::FPType::PPC_FP128 => todo!(),
            },
            _ => todo!(),
        };
        let ss = stack_allocator.alloc(num_byte);
        stack_slots.insert(name.clone(), ss);
        Ok(vec![])
    }

    pub fn build_store_inst(
        store: &llvm_ir::instruction::Store,
        stack_slots: &mut HashMap<Name, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        // dbg!(store);
        let address = &store.address;
        let val = &store.value;
        let address = Self::stack_slot_from(address, stack_slots).with_context(|| context!())?;
        // dbg!(address.gen_asm());
        let val: Operand = Self::value_from(val, regs).with_context(|| context!())?;
        // dbg!(&val);
        let mut ret: Vec<Inst> = Vec::new();
        match val {
            Operand::Imm(imm) => {
                let dst = reg_gener.gen_virtual_usual_reg();
                let li = AddInst::new(dst.into(), REG_ZERO.into(), imm.into());
                let src = dst;
                let sd = StoreInst::new(address.try_into()?, src);
                ret.push(li.into());
                ret.push(sd.into());
            }
            Operand::Fmm(_) => {
                return Err(anyhow!("store instruction with float value".to_string(),))
                    .with_context(|| context!());
            }
            Operand::Reg(reg) => {
                let src = reg;
                let sd = StoreInst::new(address.try_into()?, src);
                ret.push(sd.into());
            }
            _ => unimplemented!("store instruction with other value"),
        }
        // dbg!(&ret);
        Ok(ret)
    }

    pub fn build_load_inst(
        load: &llvm_ir::instruction::Load,
        stack_slots: &mut HashMap<Name, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        // dbg!(load);
        let mut ret: Vec<Inst> = Vec::new();
        if regs.contains_key(&load.dest) {
            unimplemented!();
        }
        let dst_reg = Self::new_var(&load.loaded_ty, reg_gener)?;
        regs.insert(load.dest.clone(), dst_reg);
        if let Ok(stack_slot) = Self::stack_slot_from(&load.address, stack_slots) {
            let ld = LoadInst::new(dst_reg, stack_slot.try_into()?);
            ret.push(ld.into());
        } else if let Ok(var) = Self::global_name_from(&load.address) {
            let addr = reg_gener.gen_virtual_usual_reg();
            let la = LaInst::new(addr, var.into());
            ret.push(la.into());
            let lw = LwInst::new(dst_reg, 0.into(), addr);
            ret.push(lw.into());
        } else {
            return Err(anyhow!("load instruction with other address".to_string()))
                .with_context(|| context!());
        }
        Ok(ret)
    }

    pub fn build_ret_imm(imm: u64) -> Imm {
        let ret_v = ((imm as i64) % 256 + 256) % 256;
        ret_v.into()
    }
    pub fn build_term_inst(
        term: &llvm_ir::Terminator,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
    ) -> Result<Vec<Inst>> {
        let mut ret_insts: Vec<Inst> = Vec::new();
        // dbg!(term);
        match term {
            llvm_ir::Terminator::Ret(r) => {
                if let Some(op) = &r.return_operand {
                    match op {
                        llvm_ir::Operand::LocalOperand { name, ty } => {
                            let reg = regs.get(name).ok_or(anyhow!("").context(context!()))?;
                            let ret_ty = ty.as_ref();
                            let mv_inst = if Self::is_ty_int(ret_ty) {
                                MvInst::new(REG_A0.into(), reg.into())
                            } else if Self::is_ty_float(ret_ty) {
                                MvInst::new(REG_FA0.into(), reg.into())
                            } else {
                                unimplemented!();
                            };
                            ret_insts.push(mv_inst.into());
                        }
                        llvm_ir::Operand::ConstantOperand(c) => match c.as_ref() {
                            Constant::Int { bits: _, value } => {
                                let imm = Self::build_ret_imm(*value);
                                let addi = AddInst::new(REG_A0.into(), REG_ZERO.into(), imm.into());
                                ret_insts.push(addi.into());
                            }
                            Constant::Float(f) => {
                                let fmm: Fmm = f.try_into()?;
                                let n = if let Some(f_var) = fmms.get(&fmm) {
                                    f_var.name.clone()
                                } else {
                                    let name = format!("_fc_{:X}", fmm.to_bits());
                                    fmms.insert(
                                        fmm.clone(),
                                        FloatVar {
                                            name: name.clone(),
                                            init: Some(fmm.try_into()?),
                                            is_const: true,
                                        },
                                    );
                                    name
                                };
                                let addr = reg_gener.gen_virtual_usual_reg();
                                let la = LaInst::new(addr, n.into());
                                ret_insts.push(la.into());
                                let loadf: Inst =
                                    if matches!(f, llvm_ir::constant::Float::Single(_)) {
                                        LwInst::new(REG_FA0, 0.into(), addr).into()
                                    } else if matches!(f, llvm_ir::constant::Float::Double(_)) {
                                        LdInst::new(REG_FA0, 0.into(), addr).into()
                                    } else {
                                        unimplemented!();
                                    };
                                ret_insts.push(loadf);
                            }
                            _ => todo!(),
                        },
                        llvm_ir::Operand::MetadataOperand => todo!(),
                    }
                }
                ret_insts.push(Inst::Ret);
            }
            llvm_ir::Terminator::CondBr(cond_br) => {
                let cond = Self::local_var_from(&cond_br.condition, regs)?;
                let true_label = Self::label_name_from(&cond_br.true_dest)?;
                let false_label = Self::label_name_from(&cond_br.false_dest)?;
                let beq = BeqInst::new(cond.try_into()?, REG_ZERO, false_label.into());
                let j = JmpInst::new(true_label.into());
                ret_insts.push(beq.into());
                ret_insts.push(j.into());
            }
            llvm_ir::Terminator::Br(br) => {
                let bb_label = Self::label_name_from(&br.dest)?;
                let j = JmpInst::new(bb_label.into());
                ret_insts.push(j.into());
            }
            _ => {
                dbg!(term);
                unimplemented!();
            }
        }

        Ok(ret_insts)
    }

    #[allow(unused)]
    pub fn build_call_inst(
        call: &llvm_ir::instruction::Call,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Name, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        let dst = &call.dest;
        let f_name = match &call.function {
            rayon::iter::Either::Left(_) => todo!(),
            rayon::iter::Either::Right(op) => {
                Self::func_name_from(op).with_context(|| context!())?
            }
        };
        let mut ret: Vec<Inst> = Vec::new();

        let mut i_arg: u32 = 0;
        let mut f_arg: u32 = 0;
        let mut extra_arg_stack: i64 = 0;
        let mut phisic_arg_regs: Vec<Reg> = Vec::new();
        for (arg, _) in &call.arguments {
            if let Ok(r) = Self::local_var_from(arg, regs) {
                let r: Reg = r.try_into()?;
                if r.is_usual() && i_arg < 8 {
                    let reg = Reg::new(REG_A0.id() + i_arg, true);
                    phisic_arg_regs.push(reg);
                    let mv = MvInst::new(reg.into(), r.into());
                    ret.push(mv.into());
                    i_arg += 1;
                } else if (!r.is_usual()) && f_arg < 8 {
                    let reg = Reg::new(REG_FA0.id() + f_arg, false);
                    phisic_arg_regs.push(reg);
                    let mv = MvInst::new(reg.into(), r.into());
                    ret.push(mv.into());
                    f_arg += 1;
                } else {
                    // extra arguments,store to stack extra_arg_stack(sp)
                    let sd = SdInst::new(r, extra_arg_stack.into(), REG_SP);
                    extra_arg_stack += 8;
                    ret.push(sd.into());
                }
            } else if let Ok(v) = Self::const_from(arg) {
                if let Some(imm) = v.imm() {
                    if i_arg < 8 {
                        let reg = Reg::new(REG_A0.id() + i_arg, true);
                        phisic_arg_regs.push(reg);
                        let li = LiInst::new(reg.into(), imm.into());
                        ret.push(li.into());
                        i_arg += 1;
                    } else {
                        let reg = reg_gener.gen_virtual_usual_reg();
                        let li = LiInst::new(reg.into(), imm.into());
                        ret.push(li.into());
                        let sd = SdInst::new(reg, extra_arg_stack.into(), REG_SP);
                        extra_arg_stack += 8;
                        ret.push(sd.into());
                    }
                } else if let Some(fmm) = v.fmm() {
                    // FIXME: fmm to reg should use other method
                    if f_arg < 8 {
                        let reg = Reg::new(REG_FA0.id() + f_arg, false);
                        phisic_arg_regs.push(reg);
                        let li = LiInst::new(reg.into(), fmm.into());
                        ret.push(li.into());
                        f_arg += 1;
                    } else {
                        let reg = reg_gener.gen_virtual_float_reg();
                        let li = LiInst::new(reg.into(), fmm.into());
                        ret.push(li.into());
                        let sd = SdInst::new(reg, extra_arg_stack.into(), REG_SP);
                        extra_arg_stack += 8;
                        ret.push(sd.into());
                    };
                } else {
                    dbg!(v);
                    unimplemented!();
                }
            } else {
                dbg!(arg);
                unimplemented!();
            }
        }

        let mut call_inst = CallInst::new(f_name.to_string().into());
        call_inst.add_uses(&phisic_arg_regs); // set reg uses for call_inst

        // 根据是否有返回值来 决定是否需要修改call_inst的defs列表
        if let Some(dest) = &call.dest {
            // with return value, add ret_reg to defs of call_inst
            // dbg!(dest);
            let func_ty = &call.function_ty;
            let dst_reg: Reg = match func_ty.as_ref() {
                llvm_ir::Type::FuncType {
                    result_type,
                    param_types,
                    is_var_arg,
                } => {
                    let (is_usual, ret_reg) = if Self::is_ty_float(result_type.as_ref()) {
                        (false, REG_FA0)
                    } else if Self::is_ty_int(result_type.as_ref()) {
                        (true, REG_A0)
                    } else {
                        unimplemented!();
                    };
                    call_inst.add_def(ret_reg);
                    ret.push(call_inst.into());
                    let dst_reg = reg_gener.gen_virtual_reg(is_usual);
                    let mv = MvInst::new(dst_reg.into(), ret_reg.into());
                    ret.push(mv.into());
                    dst_reg
                }
                _ => {
                    unimplemented!("function type");
                }
            };
            regs.insert(dest.clone(), dst_reg);
        } else {
            // if without dest value,means this call inst won't def any ret_reg
            ret.push(call_inst.into());
        }

        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_count_ret_imm() {
        fn inner_test(imm: i64, expect: Imm) {
            let imm = imm as u64;
            let ret = IRBuilder::build_ret_imm(imm);
            assert_eq!(ret, expect);
        }
        inner_test(0, 0.into());
        inner_test(1, 1.into());
        inner_test(255, 255.into());
        inner_test(256, 0.into());
        inner_test(257, 1.into());
        inner_test(-1, 255.into());
        inner_test(-250, 6.into());
        inner_test(-257, 255.into());
    }
}
