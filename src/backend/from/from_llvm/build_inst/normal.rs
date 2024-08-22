// Copyright 2024 Duskphantom Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use crate::{llvm2tac_binary_float, llvm2tac_r2_c, llvm2tac_three_op_float, llvm2tac_three_usual};

use super::*;
use llvm_ir::{Constant, Name};
use std::collections::HashMap;

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
                llvm2tac_three_usual!(AllowSwap; AddInst, add, reg_gener, regs)
            }
            llvm_ir::Instruction::Sub(sub) => Self::build_sub(sub, reg_gener, regs),
            llvm_ir::Instruction::Mul(mul) => llvm2tac_r2_c!(MulInst, mul, reg_gener, regs),
            llvm_ir::Instruction::SDiv(div) => llvm2tac_r2_c!(DivInst, div, reg_gener, regs),
            llvm_ir::Instruction::UDiv(_) => todo!(),
            llvm_ir::Instruction::URem(_) => todo!(),
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
                llvm2tac_three_usual!(AllowSwap;AndInst, and, reg_gener, regs)
            }
            llvm_ir::Instruction::Or(or) => {
                llvm2tac_three_usual!(AllowSwap;OrInst, or, reg_gener, regs)
            }
            llvm_ir::Instruction::Xor(xor) => {
                llvm2tac_three_usual!(AllowSwap;XorInst, xor, reg_gener, regs)
            }
            llvm_ir::Instruction::SRem(srem) => {
                llvm2tac_three_usual!(DenySwap;RemInst, srem, reg_gener, regs)
            }
            // process logical shift right
            llvm_ir::Instruction::LShr(lshr) => {
                llvm2tac_three_usual!(DenySwap;SrlInst, lshr, reg_gener, regs)
            }
            // process logical shift left
            llvm_ir::Instruction::Shl(shl) => {
                llvm2tac_three_usual!(DenySwap;SllInst, shl, reg_gener, regs)
            }
            // process arithmetic shift right
            llvm_ir::Instruction::AShr(ashr) => {
                llvm2tac_three_usual!(DenySwap;SraInst, ashr, reg_gener, regs)
            }
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
                Self::build_store_inst(store, stack_slots, reg_gener, regs, fmms)
            }
            llvm_ir::Instruction::SIToFP(si2f) => Self::build_si2f_inst(si2f, reg_gener, regs),
            llvm_ir::Instruction::Fence(_) => todo!(),
            llvm_ir::Instruction::CmpXchg(_) => todo!(),
            llvm_ir::Instruction::AtomicRMW(_) => todo!(),
            llvm_ir::Instruction::GetElementPtr(gep) => {
                Self::build_gep_inst(gep, stack_slots, reg_gener, regs)
            }
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
            llvm_ir::Instruction::FCmp(fcmp) => Self::build_fcmp_inst(fcmp, reg_gener, regs, fmms),
            llvm_ir::Instruction::Phi(phi) => {
                Self::build_phi_inst(phi, reg_gener, regs, insert_back_for_remove_phi)
            }
            llvm_ir::Instruction::Select(select) => {
                Self::build_select_inst(select, reg_gener, regs, fmms)
            }
            llvm_ir::Instruction::Freeze(_) => todo!(),
            llvm_ir::Instruction::Call(call) => {
                Self::build_call_inst(call, stack_slots, reg_gener, regs)
            }
            llvm_ir::Instruction::VAArg(_) => todo!(),
            llvm_ir::Instruction::LandingPad(_) => todo!(),
            llvm_ir::Instruction::CatchPad(_) => todo!(),
            llvm_ir::Instruction::CleanupPad(_) => todo!(),
        }
    }

    #[allow(unused)]
    fn build_div_inst(
        div: &llvm_ir::instruction::UDiv,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        let mut ret: Vec<Inst> = Vec::new();
        let (lhs, pre_insert) = Self::prepare_usual_lhs(&div.operand0, reg_gener, regs)?;
        ret.extend(pre_insert);
        let rhs = Self::value_from(&div.operand1, regs)?;
        let dst = reg_gener.gen_virtual_usual_reg();
        regs.insert(div.dest.clone(), dst);
        let div = DivInst::new(dst.into(), lhs, rhs);
        ret.push(div.into());
        Ok(ret)
    }

    fn build_fpext(
        fpext: &llvm_ir::instruction::FPExt,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        let src = Self::reg_from(&fpext.operand, regs)?;
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
        let (lhs, pre_insert) = Self::prepare_usual_lhs(&sub.operand0, reg_gener, regs)?;
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

    /// return (and_op, pre_insert)
    /// and_op is the operand prepared for and operation
    /// pre_insert is the instruction list for preparing and_op
    /// if flag not eq 0, and_op=(-1),which means and_op is all 1
    /// if flag eq 0, and_op=0,which means and_op is all 0
    pub fn build_and_op(
        flag: &Operand,
        reg_gener: &mut RegGenerator,
    ) -> Result<(Operand, Vec<Inst>)> {
        if let Operand::Imm(imm) = flag {
            if imm == &(0.into()) {
                return Ok((0.into(), vec![]));
            } else {
                return Ok(((-1).into(), vec![]));
            }
        }
        // 如果flag eq 0, then mid_var is 1, then and_op is 0
        // if flag not eq 0, then mid_var is 0, then and_op is -1,means all 1
        let mut insts = Vec::new();
        let mid_var = reg_gener.gen_virtual_usual_reg();
        let seqz = SeqzInst::new(mid_var.into(), flag.clone());
        insts.push(seqz.into());
        let and_op = reg_gener.gen_virtual_usual_reg();
        let add = AddInst::new(and_op.into(), mid_var.into(), (-1).into());
        insts.push(add.into());

        Ok((and_op.into(), insts))
    }

    fn build_select_inst(
        select: &llvm_ir::instruction::Select,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
    ) -> Result<Vec<Inst>> {
        fn build_and_op_from_llvm_op(
            cond: &llvm_ir::operand::Operand,
            reg_gener: &mut RegGenerator,
            regs: &HashMap<Name, Reg>,
            insts: &mut Vec<Inst>,
        ) -> Result<Operand> {
            let cond = IRBuilder::reg_from(cond, regs)?;

            let (and_op, pre_insert) = IRBuilder::build_and_op(&cond, reg_gener)?;
            insts.extend(pre_insert);

            Ok(and_op)
        }

        dbg!(select);
        // unimplemented!();
        let mut ret: Vec<Inst> = Vec::new();
        let and_op0 = build_and_op_from_llvm_op(&select.condition, reg_gener, regs, &mut ret)?;
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
            let src = Self::reg_from(&zext.operand, regs)?;
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
            let src = Self::reg_from(&sext.operand, regs)?;
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
            let (op0, prepare) = IRBuilder::prepare_usual_lhs(&icmp.operand0, reg_gener, regs)?;
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
            let (op1, prepare) = IRBuilder::prepare_usual_lhs(&icmp.operand1, reg_gener, regs)?;
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
            llvm_ir::IntPredicate::UGT => {
                let (op0, op1) = prepare_normal_op0_op1(icmp, reg_gener, regs, &mut ret)?;
                let sgtu = SgtuInst::new(dst.into(), op0.clone(), op1.clone());
                ret.push(sgtu.into());
            }
            llvm_ir::IntPredicate::UGE => {
                let (op0, op1) = prepare_normal_op0_op1(icmp, reg_gener, regs, &mut ret)?;
                let mid_var = reg_gener.gen_virtual_usual_reg();
                let sltu = SltuInst::new(mid_var.into(), op0.clone(), op1.clone());
                let xori = XorInst::new(dst.into(), mid_var.into(), 1.into());
                ret.push(sltu.into());
                ret.push(xori.into());
            }
            llvm_ir::IntPredicate::ULT => {
                let (op0, op1) = prepare_rev_op0_op1(icmp, reg_gener, regs, &mut ret)?;
                let sltu = SltuInst::new(dst.into(), op0.clone(), op1.clone());
                ret.push(sltu.into());
            }
            llvm_ir::IntPredicate::ULE => {
                let (op0, op1) = prepare_rev_op0_op1(icmp, reg_gener, regs, &mut ret)?;
                let mid_var = reg_gener.gen_virtual_usual_reg();
                let sgtu = SgtuInst::new(mid_var.into(), op0, op1);
                let xori = XorInst::new(dst.into(), mid_var.into(), 1.into());
                ret.push(sgtu.into());
                ret.push(xori.into());
            }
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

    fn build_fcmp_inst(
        fcmp: &llvm_ir::instruction::FCmp,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
    ) -> Result<Vec<Inst>> {
        let mut ret = Vec::new();
        let dst = reg_gener.gen_virtual_usual_reg();
        regs.insert(fcmp.dest.clone(), dst);

        fn prepare_op0_op1(
            fcmp: &llvm_ir::instruction::FCmp,
            reg_gener: &mut RegGenerator,
            regs: &HashMap<Name, Reg>,
            fmms: &mut HashMap<Fmm, FloatVar>,
            insts: &mut Vec<Inst>,
        ) -> Result<(Operand, Operand)> {
            let (op0, prepare) =
                IRBuilder::prepare_float_lhs(&fcmp.operand0, reg_gener, regs, fmms)?;
            insts.extend(prepare);
            let (op1, prepare) =
                IRBuilder::prepare_float_lhs(&fcmp.operand1, reg_gener, regs, fmms)?;
            insts.extend(prepare);
            Ok((op0, op1))
        }

        let (op0, op1) = prepare_op0_op1(fcmp, reg_gener, regs, fmms, &mut ret)?;
        dbg!(fcmp);
        match fcmp.predicate {
            llvm_ir::FPPredicate::False => todo!(),
            llvm_ir::FPPredicate::OEQ => {
                let feq = FeqsInst::new(dst.into(), op0, op1);
                ret.push(feq.into());
            }
            llvm_ir::FPPredicate::OLT => {
                let flt = FltsInst::new(dst.into(), op0, op1);
                ret.push(flt.into());
            }
            llvm_ir::FPPredicate::OGT => {
                let flt = FltsInst::new(dst.into(), op1, op0);
                ret.push(flt.into());
            }
            llvm_ir::FPPredicate::OGE => {
                let fle = FlesInst::new(dst.into(), op1, op0);
                ret.push(fle.into());
            }
            llvm_ir::FPPredicate::OLE => {
                let fle = FlesInst::new(dst.into(), op0, op1);
                ret.push(fle.into());
            }
            llvm_ir::FPPredicate::ONE => {
                let mid_var = reg_gener.gen_virtual_usual_reg();
                let feq = FeqsInst::new(mid_var.into(), op0, op1);
                let xori = XorInst::new(dst.into(), mid_var.into(), 1.into());
                ret.push(feq.into());
                ret.push(xori.into());
            }
            llvm_ir::FPPredicate::UNE => {
                let mid_var = reg_gener.gen_virtual_usual_reg();
                let feq = FeqsInst::new(mid_var.into(), op0, op1);
                let xori = XorInst::new(dst.into(), mid_var.into(), 1.into());
                ret.push(feq.into());
                ret.push(xori.into());
            }
            llvm_ir::FPPredicate::ORD => todo!(),
            llvm_ir::FPPredicate::UNO => todo!(),
            llvm_ir::FPPredicate::UEQ => todo!(),
            llvm_ir::FPPredicate::UGT => todo!(),
            llvm_ir::FPPredicate::UGE => todo!(),
            llvm_ir::FPPredicate::ULT => todo!(),
            llvm_ir::FPPredicate::ULE => todo!(),
            llvm_ir::FPPredicate::True => todo!(),
        }

        Ok(ret)
    }

    fn build_si2f_inst(
        si2f: &llvm_ir::instruction::SIToFP,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Vec<Inst>> {
        let op = Self::reg_from(&si2f.operand, regs)?;
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
            llvm_ir::Type::ArrayType {
                element_type: _,
                num_elements: _,
            } => {
                let e_ty = Self::basic_element_type(&ty);
                let dims = Self::dims_from_ty(&ty)?;
                let cap: usize = dims.size();
                let cap: u32 = cap.try_into().with_context(|| context!())?;
                let e_size = Self::mem_size_from(e_ty)?.num_byte();
                cap * e_size
            }
            _ => {
                dbg!(ty);
                unimplemented!();
            }
        };
        // dbg!(name.clone(), num_byte);
        let ss = stack_allocator.alloc(num_byte);
        stack_slots.insert(name.clone(), ss);
        Ok(vec![])
    }

    pub fn build_store_inst(
        store: &llvm_ir::instruction::Store,
        stack_slots: &mut HashMap<Name, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &HashMap<Name, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
    ) -> Result<Vec<Inst>> {
        // dbg!(store);
        let mut ret: Vec<Inst> = Vec::new();

        let (address, pre_insert) =
            Self::prepare_address(&store.address, reg_gener, stack_slots, regs)?;
        ret.extend(pre_insert);

        let (value, pre_insts) = Self::prepare_lhs(&store.value, reg_gener, regs, fmms)?;
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
            Operand::Reg(addr) => match value {
                Operand::Reg(val) => {
                    let sw = SwInst::new(val, 0.into(), addr);
                    ret.push(sw.into());
                }
                _ => unimplemented!("store instruction with other value"),
            },
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

        let (address, pre_insert) =
            Self::prepare_address(&load.address, reg_gener, stack_slots, regs)?;
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
                let cond = Self::reg_from(&cond_br.condition, regs)?;
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
