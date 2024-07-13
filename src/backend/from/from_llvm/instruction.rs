use super::*;
use builder::IRBuilder;
use llvm_ir::{Constant, Name};
use std::collections::HashMap;

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
            llvm_ir::Instruction::Add(_) => todo!(),
            llvm_ir::Instruction::Sub(_) => todo!(),
            llvm_ir::Instruction::Mul(_) => todo!(),
            llvm_ir::Instruction::UDiv(_) => todo!(),
            llvm_ir::Instruction::SDiv(_) => todo!(),
            llvm_ir::Instruction::URem(_) => todo!(),
            llvm_ir::Instruction::SRem(_) => todo!(),
            llvm_ir::Instruction::And(_) => todo!(),
            llvm_ir::Instruction::Or(_) => todo!(),
            llvm_ir::Instruction::Xor(_) => todo!(),
            llvm_ir::Instruction::Shl(_) => todo!(),
            llvm_ir::Instruction::LShr(_) => todo!(),
            llvm_ir::Instruction::AShr(_) => todo!(),
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
            llvm_ir::Instruction::SIToFP(_) => todo!(),
            llvm_ir::Instruction::PtrToInt(_) => todo!(),
            llvm_ir::Instruction::IntToPtr(_) => todo!(),
            llvm_ir::Instruction::BitCast(_) => todo!(),
            llvm_ir::Instruction::AddrSpaceCast(_) => todo!(),
            llvm_ir::Instruction::ICmp(_) => todo!(),
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

    /// alloca instruction only instruct allocating memory on stack,not generate one-one instruction
    fn build_alloca_inst(
        alloca: &llvm_ir::instruction::Alloca,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Name, StackSlot>,
    ) -> Result<Vec<Inst>> {
        let name = alloca.dest.clone();
        let ty = alloca.allocated_type.clone();
        let num_byte = match ty.as_ref() {
            llvm_ir::Type::IntegerType { bits } => Self::count_num_byte(*bits)?,
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
        let address = Self::address_from(address, stack_slots).with_context(|| context!())?;
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

    #[allow(unused)]
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
        let dst_reg = if Self::is_ty_int(&load.loaded_ty) {
            reg_gener.gen_virtual_usual_reg()
        } else if Self::is_ty_float(&load.loaded_ty) {
            reg_gener.gen_virtual_float_reg()
        } else {
            unimplemented!();
        };
        regs.insert(load.dest.clone(), dst_reg);
        let address = Self::address_from(&load.address, stack_slots).with_context(|| context!())?;
        let ld = LoadInst::new(dst_reg, address.try_into()?);
        ret.push(ld.into());
        Ok(ret)
    }

    pub fn build_term_inst(
        term: &llvm_ir::Terminator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        let mut ret_insts: Vec<Inst> = Vec::new();
        // dbg!(term);
        match term {
            llvm_ir::Terminator::Ret(r) => {
                if let Some(op) = &r.return_operand {
                    match op {
                        llvm_ir::Operand::LocalOperand { name, ty } => {
                            let reg = regs.get(name).ok_or(anyhow!("").context(context!()))?;
                            let mv_inst = match ty.as_ref() {
                                llvm_ir::Type::IntegerType { bits: _ } => {
                                    MvInst::new(REG_A0.into(), reg.into())
                                }
                                llvm_ir::Type::FPType(_) => MvInst::new(REG_FA0.into(), reg.into()),
                                _ => unimplemented!(),
                            };
                            ret_insts.push(mv_inst.into());
                            ret_insts.push(Inst::Ret);
                        }
                        llvm_ir::Operand::ConstantOperand(c) => match c.as_ref() {
                            Constant::Int { bits: _, value } => {
                                let imm = (*value as i64).into();
                                let addi = AddInst::new(REG_A0.into(), REG_ZERO.into(), imm);
                                ret_insts.push(addi.into());
                                ret_insts.push(Inst::Ret);
                            }
                            Constant::Float(_) => todo!(),
                            _ => todo!(),
                        },
                        llvm_ir::Operand::MetadataOperand => todo!(),
                    }
                } else {
                    unimplemented!();
                }
            }
            _ => todo!(),
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
        for (arg, _) in &call.arguments {
            if let Ok(r) = Self::local_var_from(arg, regs) {
                let r: Reg = r.try_into()?;
                let is_usual = r.is_usual();
                if r.is_usual() && i_arg < 8 {
                    let reg = Reg::new(REG_A0.id() + i_arg, true);
                    let mv = MvInst::new(reg.into(), r.into());
                    ret.push(mv.into());
                    i_arg += 1;
                } else if (!r.is_usual()) && f_arg < 8 {
                    let reg = Reg::new(REG_FA0.id() + f_arg, false);
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

        let call_inst = CallInst::new(f_name.to_string().into()).into();
        ret.push(call_inst);

        if let Some(dest) = &call.dest {
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
        }

        Ok(ret)
    }
}
