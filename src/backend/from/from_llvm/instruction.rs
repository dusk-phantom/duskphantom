use crate::{llvm2tac_binary_float, llvm2tac_three_op_float, llvm2tac_three_op_usual};

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
        fmms: &mut HashMap<Fmm, FloatVar>,
        insert_back_for_remove_phi: &mut HashMap<String, Vec<(llvm_ir::operand::Operand, Reg)>>,
    ) -> Result<Vec<Inst>> {
        // dbg!(&inst);
        match inst {
            llvm_ir::Instruction::Add(add) => {
                llvm2tac_three_op_usual!(AllowSwap; AddInst, add, reg_gener, regs)
            }
            llvm_ir::Instruction::Sub(sub) => Self::build_sub(sub, reg_gener, regs),
            llvm_ir::Instruction::Mul(mul) => {
                llvm2tac_three_op_usual!(AllowSwap; MulInst, mul, reg_gener, regs)
            }
            llvm_ir::Instruction::FAdd(fadd) => {
                llvm2tac_three_op_float!(AddInst, fadd, reg_gener, regs, fmms)
            }
            llvm_ir::Instruction::FSub(fsub) => {
                llvm2tac_three_op_float!(SubInst, fsub, reg_gener, regs, fmms)
            }
            llvm_ir::Instruction::FMul(fmul) => {
                llvm2tac_three_op_float!(MulInst, fmul, reg_gener, regs, fmms)
            }
            llvm_ir::Instruction::FDiv(fdiv) => {
                llvm2tac_three_op_float!(DivInst, fdiv, reg_gener, regs, fmms)
            }
            llvm_ir::Instruction::FRem(frem) => {
                llvm2tac_three_op_float!(RemInst, frem, reg_gener, regs, fmms)
            }
            llvm_ir::Instruction::FNeg(fneg) => {
                llvm2tac_binary_float!(NegInst, fneg, reg_gener, regs, fmms)
            }
            llvm_ir::Instruction::And(and) => {
                llvm2tac_three_op_usual!(AllowSwap;AndInst, and, reg_gener, regs)
            }
            llvm_ir::Instruction::Or(or) => {
                llvm2tac_three_op_usual!(AllowSwap;OrInst, or, reg_gener, regs)
            }
            llvm_ir::Instruction::Xor(xor) => {
                llvm2tac_three_op_usual!(AllowSwap;XorInst, xor, reg_gener, regs)
            }
            llvm_ir::Instruction::SRem(srem) => {
                llvm2tac_three_op_usual!(DenySwap;RemInst, srem, reg_gener, regs)
            }
            // process logical shift right
            llvm_ir::Instruction::LShr(lshr) => {
                llvm2tac_three_op_usual!(DenySwap;SrlInst, lshr, reg_gener, regs)
            }
            // process logical shift left
            llvm_ir::Instruction::Shl(shl) => {
                llvm2tac_three_op_usual!(DenySwap;SllInst, shl, reg_gener, regs)
            }
            // process arithmetic shift right
            llvm_ir::Instruction::AShr(ashr) => {
                llvm2tac_three_op_usual!(DenySwap;SraInst, ashr, reg_gener, regs)
            }
            llvm_ir::Instruction::UDiv(_) => todo!(),
            llvm_ir::Instruction::SDiv(_) => todo!(),
            llvm_ir::Instruction::URem(_) => todo!(),
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
            llvm_ir::Instruction::ZExt(zext) => Self::build_zext_inst(zext, reg_gener, regs),
            llvm_ir::Instruction::SExt(sext) => Self::build_sext_inst(sext, reg_gener, regs),
            llvm_ir::Instruction::FPTrunc(fptrunc) => {
                llvm2tac_binary_float!(MvInst, fptrunc, reg_gener, regs, fmms)
            }
            llvm_ir::Instruction::FPExt(fpext) => Self::build_fpext(fpext, regs),
            llvm_ir::Instruction::FPToUI(_) => todo!(),
            llvm_ir::Instruction::FPToSI(_) => todo!(),
            llvm_ir::Instruction::UIToFP(_) => todo!(),
            llvm_ir::Instruction::PtrToInt(_) => todo!(),
            llvm_ir::Instruction::IntToPtr(_) => todo!(),
            llvm_ir::Instruction::BitCast(_) => todo!(),
            llvm_ir::Instruction::AddrSpaceCast(_) => todo!(),
            llvm_ir::Instruction::ICmp(icmp) => Self::build_icmp_inst(icmp, reg_gener, regs),
            llvm_ir::Instruction::FCmp(_) => todo!(),
            llvm_ir::Instruction::Phi(phi) => {
                Self::build_phi_inst(phi, reg_gener, regs, insert_back_for_remove_phi)
            }
            llvm_ir::Instruction::Select(select) => {
                Self::build_select_inst(select, reg_gener, regs, fmms)
            }
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

    fn build_fpext(
        fpext: &llvm_ir::instruction::FPExt,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        let src = Self::local_var_from(&fpext.operand, regs)?;
        regs.insert(
            fpext.dest.clone(),
            src.try_into().with_context(|| context!())?,
        );
        Ok(vec![])
    }

    /// 翻译llvm ir的sub为riscv的sub 指令需要特别处理,因为 riscv的 sub指令不接受rhs为立即数
    fn build_sub(
        sub: &llvm_ir::instruction::Sub,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        let mut ret: Vec<Inst> = Vec::new();
        let (lhs, pre_insert) = Self::prepare_lhs(&sub.operand0, reg_gener, regs)?;
        ret.extend(pre_insert);
        let dst = reg_gener.gen_virtual_usual_reg();
        regs.insert(sub.dest.clone(), dst);

        let rhs = Self::value_from(&sub.operand1, regs)?;
        if let Operand::Imm(imm) = rhs {
            let neg_imm = -imm;
            if neg_imm.in_limit(12) {
                let add = AddInst::new(dst.into(), lhs, neg_imm.into());
                ret.push(add.into());
            } else {
                let mid_var = reg_gener.gen_virtual_usual_reg();
                let li = LiInst::new(mid_var.into(), neg_imm.into());
                let add = AddInst::new(dst.into(), lhs, mid_var.into());
                ret.push(li.into());
                ret.push(add.into());
            }
        } else if let Operand::Reg(rhs) = rhs {
            let sub = SubInst::new(dst.into(), lhs, rhs.into());
            ret.push(sub.into());
        } else {
            return Err(anyhow!("operand type not supported")).with_context(|| context!());
        }

        Ok(ret)
    }

    fn build_phi_inst(
        phi: &llvm_ir::instruction::Phi,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
        insert_back_for_remove_phi: &mut HashMap<String, Vec<(llvm_ir::operand::Operand, Reg)>>,
    ) -> Result<Vec<Inst>> {
        let dst_reg = Self::new_var(&phi.to_type, reg_gener)?;
        regs.insert(phi.dest.clone(), dst_reg);
        for (op, bb) in &phi.incoming_values {
            let bb_name = Self::label_name_from(bb)?;
            let Some(insert_backs) = insert_back_for_remove_phi.get_mut(&bb_name) else {
                let new_insert_back = vec![(op.clone(), dst_reg)];
                insert_back_for_remove_phi.insert(bb_name.clone(), new_insert_back);
                continue;
            };
            insert_backs.push((op.clone(), dst_reg));
        }
        // insert_back_for_remove_phi.insert(phi.dest.clone(), phi_regs);
        Ok(vec![])
    }

    fn build_select_inst(
        select: &llvm_ir::instruction::Select,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
    ) -> Result<Vec<Inst>> {
        fn build_and_op(
            cond: &llvm_ir::operand::Operand,
            reg_gener: &mut RegGenerator,
            regs: &HashMap<Name, Reg>,
            insts: &mut Vec<Inst>,
        ) -> Result<Operand> {
            let cond = IRBuilder::local_var_from(cond, regs)?;
            let mid_var = reg_gener.gen_virtual_usual_reg();
            let seqz = SeqzInst::new(mid_var.into(), cond);
            insts.push(seqz.into());
            let and_op = reg_gener.gen_virtual_usual_reg();
            let add = AddInst::new(and_op.into(), mid_var.into(), (-1).into());
            insts.push(add.into());
            Ok(and_op.into())
        }

        dbg!(select);
        // unimplemented!();
        let mut ret: Vec<Inst> = Vec::new();
        let and_op0 = build_and_op(&select.condition, reg_gener, regs, &mut ret)?;
        let and_op1: Operand = reg_gener.gen_virtual_usual_reg().into();
        let not = NotInst::new(and_op1.clone(), and_op0.clone());
        ret.push(not.into());

        let (true_value, pre_insts) = Self::prepare_rhs(&select.true_value, reg_gener, regs, fmms)?;
        ret.extend(pre_insts);
        let (false_value, pre_insts) =
            Self::prepare_rhs(&select.false_value, reg_gener, regs, fmms)?;
        ret.extend(pre_insts);

        let and_op0_true = reg_gener.gen_virtual_usual_reg();
        let and = AndInst::new(and_op0_true.into(), and_op0.clone(), true_value);
        ret.push(and.into());

        let and_op1_false = reg_gener.gen_virtual_usual_reg();
        let and = AndInst::new(and_op1_false.into(), and_op1.clone(), false_value);
        ret.push(and.into());

        let dst = reg_gener.gen_virtual_usual_reg();
        let or = OrInst::new(dst.into(), and_op0_true.into(), and_op1_false.into());
        regs.insert(select.dest.clone(), dst);
        ret.push(or.into());

        Ok(ret)
    }

    fn build_zext_inst(
        zext: &llvm_ir::instruction::ZExt,
        _reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        if Self::is_ty_int(&zext.to_type) {
            let src = Self::local_var_from(&zext.operand, regs)?;
            let src: Reg = src.try_into().with_context(|| context!())?;
            regs.insert(zext.dest.clone(), src);
            Ok(vec![])
        } else {
            unimplemented!();
        }
    }

    fn build_sext_inst(
        sext: &llvm_ir::instruction::SExt,
        _reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        if Self::is_ty_int(&sext.to_type) {
            let src = Self::local_var_from(&sext.operand, regs)?;
            let src: Reg = src.try_into().with_context(|| context!())?;
            regs.insert(sext.dest.clone(), src);
        } else {
            unimplemented!();
        }
        Ok(vec![])
    }

    fn build_icmp_inst(
        icmp: &llvm_ir::instruction::ICmp,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        let mut ret = Vec::new();
        let dst = reg_gener.gen_virtual_usual_reg();
        regs.insert(icmp.dest.clone(), dst);

        fn prepare_normal_op0_op1(
            icmp: &llvm_ir::instruction::ICmp,
            reg_gener: &mut RegGenerator,
            regs: &HashMap<Name, Reg>,
            insts: &mut Vec<Inst>,
        ) -> Result<(Operand, Operand)> {
            let (op0, prepare) = IRBuilder::prepare_lhs(&icmp.operand0, reg_gener, regs)?;
            insts.extend(prepare);
            let (op1, prepare) = IRBuilder::prepare_usual_rhs(&icmp.operand1, reg_gener, regs)?;
            insts.extend(prepare);
            Ok((op0, op1))
        }
        fn prepare_rev_op0_op1(
            icmp: &llvm_ir::instruction::ICmp,
            reg_gener: &mut RegGenerator,
            regs: &HashMap<Name, Reg>,
            insts: &mut Vec<Inst>,
        ) -> Result<(Operand, Operand)> {
            let (op1, prepare) = IRBuilder::prepare_lhs(&icmp.operand1, reg_gener, regs)?;
            insts.extend(prepare);
            let (op0, prepare) = IRBuilder::prepare_usual_rhs(&icmp.operand0, reg_gener, regs)?;
            insts.extend(prepare);
            Ok((op0, op1))
        }

        match icmp.predicate {
            llvm_ir::IntPredicate::EQ => {
                let (op0, op1) = prepare_normal_op0_op1(icmp, reg_gener, regs, &mut ret)?;
                let mid_var = reg_gener.gen_virtual_usual_reg();
                let xor = XorInst::new(mid_var.into(), op0.clone(), op1.clone());
                let seqz = SeqzInst::new(dst.into(), mid_var.into());
                ret.push(xor.into());
                ret.push(seqz.into());
            }
            llvm_ir::IntPredicate::NE => {
                let (op0, op1) = prepare_normal_op0_op1(icmp, reg_gener, regs, &mut ret)?;
                let mid_var = reg_gener.gen_virtual_usual_reg();
                let xor = XorInst::new(mid_var.into(), op0.clone(), op1.clone());
                let snez = SnezInst::new(dst.into(), mid_var.into());
                ret.push(xor.into());
                ret.push(snez.into());
            }
            llvm_ir::IntPredicate::UGT => todo!(),
            llvm_ir::IntPredicate::UGE => todo!(),
            llvm_ir::IntPredicate::ULT => todo!(),
            llvm_ir::IntPredicate::ULE => todo!(),
            llvm_ir::IntPredicate::SGT => {
                // notice sge(op0,op1) equal to slt(op0,op1)
                let (op0, op1) = prepare_rev_op0_op1(icmp, reg_gener, regs, &mut ret)?;
                let slt = SltInst::new(dst.into(), op1.clone(), op0.clone());
                ret.push(slt.into());
            }
            llvm_ir::IntPredicate::SGE => {
                let (op0, op1) = prepare_normal_op0_op1(icmp, reg_gener, regs, &mut ret)?;
                let mid_var = reg_gener.gen_virtual_usual_reg();
                let slt = SltInst::new(mid_var.into(), op0.clone(), op1.clone());
                // FIXME: 这里要检查一下用 xori dst,mid,1 与 用 snez dst,mid 的执行效率是否有差别
                let xori = XorInst::new(dst.into(), mid_var.into(), 1.into());
                ret.push(slt.into());
                ret.push(xori.into());
            }
            llvm_ir::IntPredicate::SLT => {
                let (op0, op1) = prepare_normal_op0_op1(icmp, reg_gener, regs, &mut ret)?;
                let slt = SltInst::new(dst.into(), op0.clone(), op1.clone());
                ret.push(slt.into());
            }
            llvm_ir::IntPredicate::SLE => {
                let (op0, op1) = prepare_rev_op0_op1(icmp, reg_gener, regs, &mut ret)?;
                let mid_var = reg_gener.gen_virtual_usual_reg();
                let slt = SltInst::new(mid_var.into(), op1.clone(), op0.clone());
                let xori = XorInst::new(dst.into(), mid_var.into(), 1.into());
                ret.push(slt.into());
                ret.push(xori.into());
            }
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
        let mut ret: Vec<Inst> = Vec::new();

        let (address, pre_insert) = Self::prepare_address(&store.address, reg_gener, stack_slots)?;
        ret.extend(pre_insert);

        let (value, pre_insts) = Self::prepare_lhs(&store.value, reg_gener, regs)?;
        ret.extend(pre_insts);

        match address {
            Operand::StackSlot(stack_slot) => match value {
                Operand::Reg(reg) => {
                    let sd = StoreInst::new(stack_slot, reg);
                    ret.push(sd.into());
                }
                _ => unimplemented!("store instruction with other value"),
            },
            Operand::Label(var) => {
                let addr = reg_gener.gen_virtual_usual_reg();
                let lla = LlaInst::new(addr, var);
                ret.push(lla.into());
                match value {
                    Operand::Reg(val) => {
                        let sw = SwInst::new(val, 0.into(), addr);
                        ret.push(sw.into());
                    }
                    _ => unimplemented!("store instruction with other value"),
                }
            }
            _ => {
                return Err(anyhow!(
                    "store instruction with invalid address {:?}",
                    address
                ))
                .with_context(|| context!());
            }
        }

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
        let dst_size = Self::mem_size_from(&load.loaded_ty)?;
        regs.insert(load.dest.clone(), dst_reg);

        let (address, pre_insert) = Self::prepare_address(&load.address, reg_gener, stack_slots)?;
        ret.extend(pre_insert);

        match address {
            Operand::StackSlot(stack_slot) => {
                let ld = match dst_size {
                    MemSize::FourByte => LoadInst::new(dst_reg, stack_slot),
                    MemSize::EightByte => LoadInst::new(dst_reg, stack_slot).with_8byte(),
                };
                ret.push(ld.into());
            }
            Operand::Label(var) => {
                let addr = reg_gener.gen_virtual_usual_reg();
                let lla = LlaInst::new(addr, var);
                ret.push(lla.into());
                let load: Inst = match dst_size {
                    MemSize::FourByte => LwInst::new(dst_reg, 0.into(), addr).into(),
                    MemSize::EightByte => LdInst::new(dst_reg, 0.into(), addr).into(),
                };
                ret.push(load);
            }
            Operand::Reg(addr) => {
                let load: Inst = match dst_size {
                    MemSize::FourByte => LwInst::new(dst_reg, 0.into(), addr).into(),
                    MemSize::EightByte => LdInst::new(dst_reg, 0.into(), addr).into(),
                };
                ret.push(load);
            }
            _ => {
                return Err(anyhow!(
                    "load instruction with invalid address {:?}",
                    address
                ))
                .with_context(|| context!());
            }
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
                                let n = Self::fmm_from_constant(c, fmms)?.name.clone();
                                let addr = reg_gener.gen_virtual_usual_reg();
                                let la = LlaInst::new(addr, n.into());
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
