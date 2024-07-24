use crate::utils::mem::ObjPtr;
use crate::{backend::*, ssa2tac_three_usual_Itype, ssa2tac_three_usual_Rtype};
use crate::{context, middle};

use crate::middle::ir::instruction::binary_inst::BinaryInst;
use crate::middle::ir::instruction::downcast_ref;
use crate::middle::ir::Instruction;

use super::*;

use anyhow::{Context, Result};

use builder::IRBuilder;
use std::collections::HashMap;
use var::FloatVar;

impl IRBuilder {
    pub fn build_instruction(
        inst: &ObjPtr<Box<dyn middle::ir::Instruction>>,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Address, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
        insert_back_for_remove_phi: &mut HashMap<String, Vec<(middle::ir::Operand, Reg)>>,
    ) -> Result<Vec<Inst>> {
        match inst.get_type() {
            middle::ir::instruction::InstType::Head => {
                Err(anyhow!("head should not be in backend")).with_context(|| context!())
            } // 应该是不能有 Head 出现的
            middle::ir::instruction::InstType::Alloca => {
                let alloca = downcast_ref::<middle::ir::instruction::memory_op_inst::Alloca>(
                    inst.as_ref().as_ref(),
                );
                Self::build_alloca_inst(alloca, stack_allocator, stack_slots)
            }
            middle::ir::instruction::InstType::Store => {
                let store = downcast_ref::<middle::ir::instruction::memory_op_inst::Store>(
                    inst.as_ref().as_ref(),
                );
                Self::build_store_inst(store, stack_slots, reg_gener, regs)
            }
            middle::ir::instruction::InstType::Add => {
                ssa2tac_three_usual_Itype!(AddInst, Add, inst, regs, reg_gener)
            }
            middle::ir::instruction::InstType::FAdd => {
                // ssa2tac_binary_float!(inst, regs, reg_gener, FAdd, Add, AddInst)
                let mut insts = Vec::new();
                let fadd = downcast_ref::<middle::ir::instruction::binary_inst::FAdd>(
                    inst.as_ref().as_ref(),
                );
                let (op0, prepare) = Self::prepare_f(fadd.get_lhs(), reg_gener, regs, fmms)?;
                insts.extend(prepare);
                let (op1, prepare) = Self::prepare_f(fadd.get_rhs(), reg_gener, regs, fmms)?;
                insts.extend(prepare);
                let dst0 = reg_gener.gen_virtual_float_reg();
                let add_inst = AddInst::new(dst0.into(), op0, op1);
                regs.insert(fadd as *const _ as Address, dst0);
                insts.push(add_inst.into());
                Ok(insts)
            }
            middle::ir::instruction::InstType::Sub => {
                ssa2tac_three_usual_Rtype!(SubInst, Sub, inst, regs, reg_gener)
            }
            // 通过类型转换，可以做到: FAdd 的输入一定是 Float 类型的寄存器
            middle::ir::instruction::InstType::FSub => {
                // ssa2tac_binary_float!(inst, regs, reg_gener, FSub, Sub, SubInst)
                todo!();
            }
            middle::ir::instruction::InstType::Mul => {
                ssa2tac_three_usual_Rtype!(MulInst, Mul, inst, regs, reg_gener)
            }
            middle::ir::instruction::InstType::FMul => {
                // ssa2tac_binary_float!(inst, regs, reg_gener, FMul, Mul, MulInst)
                todo!()
            }
            middle::ir::instruction::InstType::SDiv => {
                ssa2tac_three_usual_Rtype!(DivInst, SDiv, inst, regs, reg_gener)
            }
            middle::ir::instruction::InstType::SRem => {
                ssa2tac_three_usual_Rtype!(RemInst, SRem, inst, regs, reg_gener)
            }
            middle::ir::instruction::InstType::UDiv => todo!(), // TODO 目前还没有 udiv 和 urem
            middle::ir::instruction::InstType::URem => todo!(),
            middle::ir::instruction::InstType::FDiv => todo!(),
            middle::ir::instruction::InstType::Shl => {
                ssa2tac_three_usual_Itype!(SllInst, Shl, inst, regs, reg_gener)
            }
            middle::ir::instruction::InstType::LShr => {
                ssa2tac_three_usual_Itype!(SrlInst, LShr, inst, regs, reg_gener)
            }
            middle::ir::instruction::InstType::AShr => {
                // ssa2tac_binary_usual!(SraInst, AShr, inst, regs, reg_gener)
                // .into() 会报错
                ssa2tac_three_usual_Itype!(SraInst, AShr, inst, regs, reg_gener)
            }
            middle::ir::instruction::InstType::And => {
                ssa2tac_three_usual_Itype!(AndInst, And, inst, regs, reg_gener)
            }
            middle::ir::instruction::InstType::Or => {
                ssa2tac_three_usual_Itype!(OrInst, Or, inst, regs, reg_gener)
            }
            middle::ir::instruction::InstType::Xor => {
                ssa2tac_three_usual_Itype!(XorInst, Xor, inst, regs, reg_gener)
            }
            middle::ir::instruction::InstType::Ret => {
                let ret = downcast_ref::<middle::ir::instruction::terminator_inst::Ret>(
                    inst.as_ref().as_ref(),
                );
                Self::build_ret_inst(ret, reg_gener, regs, fmms)
            }
            middle::ir::instruction::InstType::Br => {
                let br = downcast_ref::<middle::ir::instruction::terminator_inst::Br>(
                    inst.as_ref().as_ref(),
                );
                Self::build_br_inst(br, regs)
            }
            middle::ir::instruction::InstType::Load => {
                let load = downcast_ref::<middle::ir::instruction::memory_op_inst::Load>(
                    inst.as_ref().as_ref(),
                );
                Self::build_load_inst(load, stack_slots, reg_gener, regs)
            }
            middle::ir::instruction::InstType::GetElementPtr => {
                let gep = downcast_ref::<middle::ir::instruction::memory_op_inst::GetElementPtr>(
                    inst.as_ref().as_ref(),
                );
                Self::build_gep_inst(gep, reg_gener, regs)
            }
            middle::ir::instruction::InstType::ZextTo => {
                let zext = downcast_ref::<middle::ir::instruction::extend_inst::ZextTo>(
                    inst.as_ref().as_ref(),
                );
                Self::build_zext_inst(zext, reg_gener, regs)
            }
            middle::ir::instruction::InstType::SextTo => todo!(),
            middle::ir::instruction::InstType::ItoFp => {
                let itofp = downcast_ref::<middle::ir::instruction::extend_inst::ItoFp>(
                    inst.as_ref().as_ref(),
                );
                let src = Self::value_from(itofp.get_src(), regs).with_context(|| context!())?;
                let dst = reg_gener.gen_virtual_float_reg();
                let fcvtsw = I2fInst::new(dst.into(), src); // FIXME 不过我对这里有点疑惑: 中端会不会给浮点型立即数, 然后浮点型立即数实际上也需要特殊处理
                regs.insert(itofp as *const _ as Address, dst);
                Ok(vec![fcvtsw.into()])
            }
            middle::ir::instruction::InstType::FpToI => {
                let fptoi = downcast_ref::<middle::ir::instruction::extend_inst::FpToI>(
                    inst.as_ref().as_ref(),
                );
                let src = Self::value_from(fptoi.get_src(), regs).with_context(|| context!())?;
                let dst = reg_gener.gen_virtual_usual_reg();
                let fcvtws = I2fInst::new(dst.into(), src); //
                regs.insert(fptoi as *const _ as Address, dst);
                Ok(vec![fcvtws.into()])
            }
            middle::ir::instruction::InstType::ICmp => {
                let icmp = downcast_ref::<middle::ir::instruction::misc_inst::ICmp>(
                    inst.as_ref().as_ref(),
                );
                Self::build_icmp_inst(icmp, reg_gener, regs)
            }
            middle::ir::instruction::InstType::FCmp => todo!(),
            middle::ir::instruction::InstType::Phi => {
                let phi =
                    downcast_ref::<middle::ir::instruction::misc_inst::Phi>(inst.as_ref().as_ref());
                Self::build_phi_inst(phi, reg_gener, regs, insert_back_for_remove_phi)
            }
            middle::ir::instruction::InstType::Call => {
                let call = downcast_ref::<middle::ir::instruction::misc_inst::Call>(
                    inst.as_ref().as_ref(),
                );
                Self::build_call_inst(call, stack_allocator, stack_slots, reg_gener, regs)
            }
        }
    }

    // fn build_select_inst(  // 我们中端没有 Select
    //     select: &middle::ir::instruction,
    //     reg_gener: &mut RegGenerator,
    //     regs: &mut HashMap<Address, Reg>,
    // ) -> Result<Vec<Inst>> {
    //     todo!()
    // }

    #[allow(unused)]
    fn build_gep_inst(
        gep: &middle::ir::instruction::memory_op_inst::GetElementPtr,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
    ) -> Result<Vec<Inst>> {
        todo!()
    }

    fn build_phi_inst(
        phi: &middle::ir::instruction::misc_inst::Phi,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
        insert_back_for_remove_phi: &mut HashMap<String, Vec<(middle::ir::Operand, Reg)>>,
    ) -> Result<Vec<Inst>> {
        let dst_reg = Self::new_var(&phi.get_value_type(), reg_gener)?;
        regs.insert(phi as *const _ as Address, dst_reg);
        for (op, bb) in phi.get_incoming_values() {
            let bb_name = Self::label_name_from(bb);
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

    #[allow(unused)]
    fn build_zext_inst(
        zext: &middle::ir::instruction::extend_inst::ZextTo,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
    ) -> Result<Vec<Inst>> {
        match zext.get_src() {
            middle::ir::Operand::Constant(_) => todo!(),
            middle::ir::Operand::Global(_) => todo!(),
            middle::ir::Operand::Parameter(_) => todo!(),
            middle::ir::Operand::Instruction(instr) => {
                let src = Self::local_var_from(instr, regs).with_context(|| context!())?;
                let dst = reg_gener.gen_virtual_usual_reg();
                regs.insert(zext as *const _ as Address, dst);
                let xt = AndInst::new(dst.into(), src.into(), (-1).into());
                Ok(vec![xt.into()])
            }
        }
    }

    fn build_icmp_inst(
        icmp: &middle::ir::instruction::misc_inst::ICmp,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
    ) -> Result<Vec<Inst>> {
        /* ---------- 辅助函数 ---------- */
        fn prepare_normal_op0_op1(
            icmp: &middle::ir::instruction::misc_inst::ICmp,
            reg_gener: &mut RegGenerator,
            regs: &HashMap<Address, Reg>,
            insts: &mut Vec<Inst>,
        ) -> Result<(Operand, Operand)> {
            let lhs = icmp.get_lhs();
            let rhs = icmp.get_rhs();
            let (op0, prepare) = IRBuilder::prepare_rs1_i(lhs, reg_gener, regs)?;
            insts.extend(prepare);
            let (op1, prepare) = IRBuilder::prepare_rs2_i(rhs, reg_gener, regs)?;
            insts.extend(prepare);
            Ok((op0, op1))
        }
        fn prepare_rev_op0_op1(
            icmp: &middle::ir::instruction::misc_inst::ICmp,
            reg_gener: &mut RegGenerator,
            regs: &HashMap<Address, Reg>,
            insts: &mut Vec<Inst>,
        ) -> Result<(Operand, Operand)> {
            let lhs = icmp.get_lhs();
            let rhs = icmp.get_rhs();
            let (op0, prepare) = IRBuilder::prepare_rs1_i(rhs, reg_gener, regs)?;
            insts.extend(prepare);
            let (op1, prepare) = IRBuilder::prepare_rs2_i(lhs, reg_gener, regs)?;
            insts.extend(prepare);
            Ok((op0, op1))
        }

        /* ----------  ---------- */

        // let mut ret = Vec::new();
        let flag = reg_gener.gen_virtual_usual_reg();
        regs.insert(icmp as *const _ as Address, flag);

        let mut ret = Vec::new();

        match icmp.op {
            middle::ir::instruction::misc_inst::ICmpOp::Eq => {
                // a == b <=> a ^ b == 0
                let (op0, op1) = prepare_normal_op0_op1(icmp, reg_gener, regs, &mut ret)?;
                let _mid = reg_gener.gen_virtual_usual_reg();
                let xor = XorInst::new(_mid.into(), op0.clone(), op1.clone());
                let seqz = SeqzInst::new(flag.into(), _mid.into());
                ret.push(xor.into());
                ret.push(seqz.into());
            }
            middle::ir::instruction::misc_inst::ICmpOp::Ne => {
                // a != b <=> a ^ b != 0
                let (op0, op1) = prepare_normal_op0_op1(icmp, reg_gener, regs, &mut ret)?;
                let _mid = reg_gener.gen_virtual_usual_reg();
                let xor = XorInst::new(_mid.into(), op0.clone(), op1.clone());
                let snez = SnezInst::new(flag.into(), _mid.into());
                ret.push(xor.into());
                ret.push(snez.into());
            }
            middle::ir::instruction::misc_inst::ICmpOp::Slt => {
                // a < b
                let (op0, op1) = prepare_normal_op0_op1(icmp, reg_gener, regs, &mut ret)?;
                let slt = SltInst::new(flag.into(), op0, op1);
                ret.push(slt.into());
            }
            middle::ir::instruction::misc_inst::ICmpOp::Sle => {
                // lhs <= rhs <=> ~(lhs > rhs) <=> (lhs > rhs) == 0 <=> (rhs < lhs) == 0 === (op0 < op1) == 0
                let (op0, op1) = prepare_rev_op0_op1(icmp, reg_gener, regs, &mut ret)?;
                let _mid = reg_gener.gen_virtual_usual_reg();
                let slt = SltInst::new(_mid.into(), op0, op1);
                let seqz = SeqzInst::new(flag.into(), _mid.into());
                ret.push(slt.into());
                ret.push(seqz.into());
            }
            middle::ir::instruction::misc_inst::ICmpOp::Sgt => {
                // lhs > rhs <=> rhs < lhs <=> op0 < op1
                let (op0, op1) = prepare_rev_op0_op1(icmp, reg_gener, regs, &mut ret)?;
                let slt = SltInst::new(flag.into(), op0, op1);
                ret.push(slt.into());
            }
            middle::ir::instruction::misc_inst::ICmpOp::Sge => {
                // op0 >= op1 <=> ~(op0 < op1) <=> (op0 < op1) == 0
                let (op0, op1) = prepare_normal_op0_op1(icmp, reg_gener, regs, &mut ret)?;
                let _mid = reg_gener.gen_virtual_usual_reg();
                let slt = SltInst::new(_mid.into(), op0, op1);
                let seqz = SeqzInst::new(flag.into(), _mid.into());
                ret.push(slt.into());
                ret.push(seqz.into());
            }
            middle::ir::instruction::misc_inst::ICmpOp::Ult => todo!(),
            middle::ir::instruction::misc_inst::ICmpOp::Ule => todo!(),
            middle::ir::instruction::misc_inst::ICmpOp::Ugt => todo!(),
            middle::ir::instruction::misc_inst::ICmpOp::Uge => todo!(),
        }
        Ok(ret)
    }

    /// alloca instruction only instruct allocating memory on stack,not generate one-one instruction
    fn build_alloca_inst(
        alloca: &middle::ir::instruction::memory_op_inst::Alloca,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Address, StackSlot>,
    ) -> Result<Vec<Inst>> {
        let ty = alloca.value_type.clone();
        let bytes: u32 = match ty {
            middle::ir::ValueType::Int => 8,
            middle::ir::ValueType::Void => {
                return Err(anyhow!("it can't alloca void")).with_context(|| context!())
            }
            middle::ir::ValueType::Float => 8,
            middle::ir::ValueType::Bool => 8,
            middle::ir::ValueType::SignedChar => 8,
            middle::ir::ValueType::Array(_, _) => todo!(),
            middle::ir::ValueType::Pointer(_) => todo!(), // 4B
                                                          // _ => todo!(),
                                                          // TODO 如果是其他大小的指令
        };
        let ss = stack_allocator.alloc(bytes);
        stack_slots.insert(
            alloca as *const _ as Address, /* alloca 的目的寄存器, 里面存放有栈上变量的地址 */
            ss,                            /* 栈上分配的地址 */
        ); /* 将 栈上地址 与 目的寄存器 关联起来 */
        Ok(vec![])
    }

    /// store 指令，有几种可能: 指针/数组、全局变量、栈
    pub fn build_store_inst(
        store: &middle::ir::instruction::memory_op_inst::Store,
        stack_slots: &mut HashMap<Address, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &HashMap<Address, Reg>,
    ) -> Result<Vec<Inst>> {
        // 这个 address
        // 有三种来源: 1. 全局变量 2. alloca 3. get_element_ptr // TODO 目前只有 stack
        let address: &&middle::ir::Operand = &store.get_ptr();
        // address 这个地址是 stack 上的地址
        let address = Self::stack_slot_from(address, stack_slots).with_context(|| context!())?;
        let val = &store.get_value();
        let val = Self::value_from(val, regs).with_context(|| context!())?;
        let mut ret: Vec<Inst> = Vec::new();
        match val {
            Operand::Imm(imm) => {
                let _val = reg_gener.gen_virtual_usual_reg(); // 分配一个临时的 dest, 用来存储 imm, 因此 sd reg, stack_slot
                let li = LiInst::new(_val.into(), imm.into()); // li dst, imm
                let sd = StoreInst::new(address.try_into()?, _val); // sd src, stack_slot
                ret.push(li.into());
                ret.push(sd.into());
            }
            Operand::Fmm(_) => {
                return Err(anyhow!("store instruction with float value".to_string(),))
                    .with_context(|| context!());
            }
            Operand::Reg(re) => {
                let sd = StoreInst::new(address.try_into()?, re); // sd src, stack_slot
                ret.push(sd.into());
            }
            // Operand::StackSlot(_)
            // Operand::Label(_)
            _ => todo!(),
        }
        Ok(ret)
    }

    pub fn build_load_inst(
        load: &middle::ir::instruction::memory_op_inst::Load,
        stack_slots: &mut HashMap<Address, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
    ) -> Result<Vec<Inst>> {
        // dbg!(load);
        let mut ret: Vec<Inst> = Vec::new();
        if regs.contains_key(&(load as *const _ as Address)) {
            unimplemented!() // 已经 load 过一次了
        }
        let dst_reg = match load.get_value_type() {
            middle::ir::ValueType::Int => reg_gener.gen_virtual_usual_reg(),
            middle::ir::ValueType::Float => reg_gener.gen_virtual_float_reg(),
            middle::ir::ValueType::Bool => reg_gener.gen_virtual_usual_reg(),
            _ => {
                /* void, array, pointer */
                return Err(anyhow!("load instruction with array/pointer/void"))
                    .with_context(|| context!());
            }
        };
        regs.insert(load as *const _ as Address, dst_reg);
        // 两种情况: 1. 从栈上获取(之前 alloca 过一次), 2. 从非栈上获取(parameter-pointer, global)
        if let Ok(slot) = Self::stack_slot_from(load.get_ptr(), stack_slots) {
            let ld = LoadInst::new(dst_reg, slot.try_into()?);
            ret.push(ld.into());
        } else if let Ok(label) = Self::global_from(load.get_ptr()) {
            let addr = reg_gener.gen_virtual_usual_reg();
            let la = LlaInst::new(addr, label.into());
            ret.push(la.into());
            let lw = LwInst::new(dst_reg, 0.into(), addr);
            ret.push(lw.into());
        } else if let Ok(base /* 基地址 */) = Self::pointer_from(load.get_ptr(), regs) {
            let lw = LwInst::new(dst_reg, 0.into(), base);
            ret.push(lw.into());
        } else {
            return Err(anyhow!("load instruction with other address")).with_context(|| context!());
        }
        Ok(ret)
    }

    pub fn build_ret_inst(
        ret: &middle::ir::instruction::terminator_inst::Ret,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
    ) -> Result<Vec<Inst>> {
        let mut ret_insts: Vec<Inst> = Vec::new();

        // 准备返回值
        if !ret.is_void() {
            let op = ret.get_return_value();
            match op {
                middle::ir::Operand::Constant(c) => match c {
                    middle::ir::Constant::SignedChar(c) => {
                        let imm: Operand = (*c as i64).into();
                        let li = AddInst::new(REG_A0.into(), REG_ZERO.into(), imm);
                        ret_insts.push(li.into());
                    }
                    middle::ir::Constant::Int(i) => {
                        let imm: Operand = (*i as i64).into();
                        let li = AddInst::new(REG_A0.into(), REG_ZERO.into(), imm);
                        ret_insts.push(li.into());
                    }
                    middle::ir::Constant::Bool(b) => {
                        let imm: Operand = (*b as i64).into();
                        let li = AddInst::new(REG_A0.into(), REG_ZERO.into(), imm);
                        ret_insts.push(li.into());
                    }
                    middle::ir::Constant::Float(f) => {
                        let fmm: Fmm = f.into();
                        let n = if let Some(f_var) = fmms.get(&fmm) {
                            f_var.name.clone() // 这个 name 是我们自己加进去的
                        } else {
                            let name = format!("_fc_{:x}", fmm.to_bits());
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
                        let la = LlaInst::new(addr, n.into());
                        ret_insts.push(la.into());
                        // 不过这里没有 double
                        let loadf = LwInst::new(REG_FA0, 0.into(), addr);
                        ret_insts.push(loadf.into());
                    }
                    middle::ir::Constant::Array(_) => {
                        return Err(anyhow!("return array is not allow:{}", op))
                            .with_context(|| context!())
                    }
                },
                middle::ir::Operand::Instruction(instr) => {
                    let addr = instr.as_ref().as_ref() as *const dyn middle::ir::Instruction
                        as *const () as Address;
                    let reg = regs
                        .get(&addr)
                        .ok_or(anyhow!("could not get {} from map", &addr).context(context!()))?; // 获取返回值对应的虚拟寄存器
                    let mv_inst = match instr.get_value_type() {
                        middle::ir::ValueType::Int
                        | middle::ir::ValueType::Bool
                        | middle::ir::ValueType::SignedChar => {
                            MvInst::new(REG_A0.into(), (*reg).into())
                        }
                        middle::ir::ValueType::Float => MvInst::new(REG_FA0.into(), (*reg).into()),
                        middle::ir::ValueType::Void => {
                            return Err(anyhow!("return not is_void, but get void type"))
                                .with_context(|| context!())
                        }
                        middle::ir::ValueType::Array(_, _) => {
                            return Err(anyhow!("return array is not allow for sysy"))
                                .with_context(|| context!())
                        }
                        middle::ir::ValueType::Pointer(_) => {
                            // NOTE 注意一下这里 可能可以返回指针
                            return Err(anyhow!("return pointer is not allow for sysy"))
                                .with_context(|| context!());
                        }
                    };
                    ret_insts.push(mv_inst.into());
                }
                middle::ir::Operand::Global(glo) => {
                    return Err(anyhow!("return global should be load first :{}", glo))
                        .with_context(|| context!())
                }
                middle::ir::Operand::Parameter(param) => {
                    let addr = param.as_ref() as *const _ as Address;
                    let reg = regs.get(&addr).ok_or(anyhow!("").context(context!()))?;
                    let mv_inst = match param.value_type {
                        middle::ir::ValueType::Void => {
                            return Err(anyhow!("return not is_void, but get void type"))
                                .with_context(|| context!())
                        }
                        middle::ir::ValueType::Int
                        | middle::ir::ValueType::Bool
                        | middle::ir::ValueType::SignedChar => {
                            MvInst::new(REG_A0.into(), (*reg).into())
                        }
                        middle::ir::ValueType::Float => MvInst::new(REG_FA0.into(), (*reg).into()),
                        middle::ir::ValueType::Array(_, _) => {
                            return Err(anyhow!("return array is not allow for sysy"))
                                .with_context(|| context!())
                        }
                        middle::ir::ValueType::Pointer(_) => {
                            return Err(anyhow!("return pointer is not allow for sysy"))
                                .with_context(|| context!())
                        }
                    };
                    ret_insts.push(mv_inst.into());
                }
            }
        } /*  else {  // 如果返回值是 void, 那么啥也不应管了
          } */

        // 最后的一条 ret
        ret_insts.push(Inst::Ret);
        Ok(ret_insts)
    }

    pub fn build_br_inst(
        br: &middle::ir::instruction::terminator_inst::Br,
        regs: &mut HashMap<Address, Reg>,
    ) -> Result<Vec<Inst>> {
        let parent_bb = br
            .get_parent_bb()
            .ok_or(anyhow!("get parent bb failed"))
            .with_context(|| context!())?;
        let mut br_insts: Vec<Inst> = Vec::new();
        if br.is_cond_br() {
            let (cond, instrs) =
                Self::prepare_cond(br.get_cond(), regs).with_context(|| context!())?;
            br_insts.extend(instrs);
            let true_bb = parent_bb
                .get_succ_bb()
                .first()
                .ok_or(anyhow!("get true bb failed"))
                .with_context(|| context!())?;
            let true_label = Self::label_name_from(true_bb);
            let false_bb = parent_bb
                .get_succ_bb()
                .get(1)
                .ok_or(anyhow!("get false bb failed"))
                .with_context(|| context!())?;
            let false_label = Self::label_name_from(false_bb);
            let beqz = BeqInst::new(cond, REG_ZERO, false_label.into());
            let j = JmpInst::new(true_label.into());
            br_insts.push(beqz.into());
            br_insts.push(j.into());
        } else {
            let succ = parent_bb
                .get_succ_bb()
                .first()
                .ok_or(anyhow!("get succ bb failed"))
                .with_context(|| context!())?;
            let succ_label = Self::label_name_from(succ);
            let j = JmpInst::new(succ_label.into());
            br_insts.push(j.into());
        }

        Ok(br_insts)
    }

    // /// 不是 ret 就是 br
    // #[allow(unused)]
    // pub fn build_term_inst(
    //     term: &ObjPtr<Box<dyn middle::ir::Instruction>>,
    //     regs: &mut HashMap<Address, Reg>,
    //     reg_gener: &mut RegGenerator,
    //     fmms: &mut HashMap<Fmm, FloatVar>,
    // ) -> Result<Vec<Inst>> {
    //     let mut ret_insts: Vec<Inst> = Vec::new();
    //     // dbg!(term);

    //     let insts = match term.get_type() {
    //         middle::ir::instruction::InstType::Ret => {
    //             let ret = downcast_ref::<middle::ir::instruction::terminator_inst::Ret>(
    //                 term.as_ref().as_ref(),
    //             );
    //             Self::build_ret_inst(ret, reg_gener, regs, fmms)?
    //         }
    //         middle::ir::instruction::InstType::Br => {
    //             let br = downcast_ref::<middle::ir::instruction::terminator_inst::Br>(
    //                 term.as_ref().as_ref(),
    //             );
    //             Self::build_br_inst(br, regs)?
    //         }
    //         _ => {
    //             return Err(anyhow!("get_last_inst only to be ret or br"))
    //                 .with_context(|| context!())
    //         }
    //     };

    //     // TODO 这里 return 还要记得 退栈

    //     ret_insts.extend(insts);

    //     Ok(ret_insts)
    // }

    #[allow(unused)]
    pub fn build_call_inst(
        // call: &llvm_ir::instruction::Call,
        call: &middle::ir::instruction::misc_inst::Call,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Address, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
    ) -> Result<Vec<Inst>> {
        let mut call_insts: Vec<Inst> = Vec::new(); // build_call_inst 的返回值

        /* ---------- 参数 ---------- */

        let mut i_arg_num: u32 = 0;
        let mut f_arg_num: u32 = 0;
        let mut extra_arg_stack: i64 = 0;
        let mut phisic_arg_regs: Vec<Reg> = Vec::new();
        let arguments = call.get_operand(); // 参数列表, 这个可以类比成 llvm_ir::call::arguments
        for arg in arguments {
            let ope = Self::value_from(arg, regs).context(context!())?;
            match ope {
                Operand::Reg(r) => {
                    if r.is_usual() && i_arg_num < 8 {
                        let reg = Reg::new(REG_A0.id() + i_arg_num, true);
                        phisic_arg_regs.push(reg);
                        let mv = MvInst::new(reg.into(), ope);
                        call_insts.push(mv.into());
                        i_arg_num += 1;
                    } else if (!r.is_usual()) && f_arg_num < 8 {
                        let reg = Reg::new(REG_FA0.id() + f_arg_num, false);
                        phisic_arg_regs.push(reg);
                        let mv = MvInst::new(reg.into(), ope);
                        call_insts.push(mv.into());
                        f_arg_num += 1;
                    } else {
                        let sd = SdInst::new(r, extra_arg_stack.into(), REG_SP);
                        extra_arg_stack += 8;
                        call_insts.push(sd.into());
                    }
                }
                Operand::Imm(imm) => {
                    if i_arg_num < 8 {
                        let reg = Reg::new(REG_A0.id() + i_arg_num, true);
                        let li = LiInst::new(reg.into(), imm.into());
                        phisic_arg_regs.push(reg);
                        call_insts.push(li.into());
                        i_arg_num += 1;
                    } else {
                        let reg = Reg::new(REG_A0.id() + i_arg_num, true);
                        let li = LiInst::new(reg.into(), imm.into());
                        call_insts.push(li.into());
                        let sd = SdInst::new(reg, extra_arg_stack.into(), REG_SP);
                        extra_arg_stack += 8;
                        call_insts.push(sd.into());
                    }
                }
                Operand::Fmm(fmm) => {
                    if f_arg_num < 8 {
                        // TODO 不一定是 li, 这里可能有问题, 可能是 flw/fld 之类的
                        let reg = Reg::new(REG_FA0.id() + f_arg_num, false);
                        let li = LiInst::new(reg.into(), fmm.into());
                        phisic_arg_regs.push(reg);
                        call_insts.push(li.into());
                        f_arg_num += 1;
                    } else {
                        let reg = Reg::new(REG_FA0.id() + f_arg_num, false);
                        let li = LiInst::new(reg.into(), fmm.into());
                        call_insts.push(li.into());
                        let sd = SdInst::new(reg, extra_arg_stack.into(), REG_SP);
                        extra_arg_stack += 8;
                        call_insts.push(sd.into());
                    }
                }
                Operand::StackSlot(_) => todo!(), // TODO 这个有待商榷
                Operand::Label(_) => {
                    return Err(anyhow!("argument can't be a label".to_string()))
                        .with_context(|| context!())
                }
            }
        }

        /* ---------- call 指令本身 ---------- */

        // 函数是全局的，因此用的是名字
        let mut call_inst: CallInst = CallInst::new(call.func.name.to_string().into()); // call <一个全局的 name >

        let dest_name = call as *const _ as Address;

        let func = call.func;

        /* ---------- 返回值 ---------- */

        // call 返回之后，将返回值放到一个虚拟寄存器中
        match func.return_type {
            middle::ir::ValueType::Void => {
                call_insts.push(call_inst.into());
            }
            middle::ir::ValueType::Int
            | middle::ir::ValueType::Float
            | middle::ir::ValueType::Bool => {
                let is_usual = func.return_type == middle::ir::ValueType::Int
                    || func.return_type == middle::ir::ValueType::Bool;
                let dst = if is_usual {
                    reg_gener.gen_virtual_usual_reg()
                } else {
                    reg_gener.gen_virtual_float_reg()
                }; // 分配一个虚拟寄存器
                let ret_reg = if is_usual { REG_A0 } else { REG_FA0 };
                let mv = MvInst::new(dst.into(), ret_reg.into());
                regs.insert(dest_name, dst); // 绑定中端的 id 和 虚拟寄存器

                // 有返回值的情况下,传递返回值的ret_reg寄存器被认为被这条call指令
                // 定义了,需要加入到该指令的defs列表中
                call_inst.add_def(ret_reg);
                call_insts.push(call_inst.into());
                call_insts.push(mv.into());
            }
            _ => {
                return Err(anyhow!("sysy only return: void | float | int".to_string()))
                    .with_context(|| context!())
            }
        };

        Ok(call_insts)
    }
}

mod tests {
    #[allow(unused)]
    use crate::{
        backend::from::from_self::Address,
        middle::{
            self,
            ir::{instruction::downcast_ref, IRBuilder, Instruction, ValueType},
        },
    };

    /// 测试地址是否改变
    #[test]
    fn test_address() {
        let mut ir_builder = IRBuilder::new();
        let ptr = ir_builder.get_alloca(ValueType::Int, 1);
        let load_0 = ir_builder.get_load(ValueType::Int, middle::ir::Operand::Instruction(ptr));

        let ss = load_0.as_ref().as_ref() as *const dyn Instruction as *const () as Address;
        dbg!(&ss);

        let inst: &middle::ir::instruction::memory_op_inst::Load =
            downcast_ref::<middle::ir::instruction::memory_op_inst::Load>(load_0.as_ref().as_ref());

        let address = inst as *const middle::ir::instruction::memory_op_inst::Load as Address;
        dbg!(&address);

        after_downcast(inst);
    }

    #[allow(dead_code)]
    fn after_downcast(inst: &middle::ir::instruction::memory_op_inst::Load) {
        let address = inst as *const middle::ir::instruction::memory_op_inst::Load as Address;
        dbg!(&address);
    }
}
