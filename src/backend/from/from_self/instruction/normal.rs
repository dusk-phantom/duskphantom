use super::*;

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
                Self::build_store_inst(store, stack_slots, reg_gener, regs, fmms)
            }
            middle::ir::instruction::InstType::Add => {
                ssa2tac_three_usual_Itype!(AddInst, Add, inst, regs, reg_gener)
            }
            middle::ir::instruction::InstType::FAdd => {
                // ssa2tac_three_float!(inst, regs, reg_gener, FAdd, Add, AddInst)
                ssa2tac_three_float!(AddInst, FAdd, inst, regs, reg_gener, fmms)
                // let mut insts = Vec::new();
                // let fadd = downcast_ref::<middle::ir::instruction::binary_inst::FAdd>(
                //     inst.as_ref().as_ref(),
                // );
                // let (op0, prepare) = Self::prepare_f(fadd.get_lhs(), reg_gener, regs, fmms)?;
                // insts.extend(prepare);
                // let (op1, prepare) = Self::prepare_f(fadd.get_rhs(), reg_gener, regs, fmms)?;
                // insts.extend(prepare);
                // let dst0 = reg_gener.gen_virtual_float_reg();
                // let fadd_inst = AddInst::new(dst0.into(), op0, op1);
                // regs.insert(fadd as *const _ as Address, dst0);
                // insts.push(fadd_inst.into());
                // Ok(insts)
            }
            middle::ir::instruction::InstType::Sub => {
                ssa2tac_three_usual_Rtype!(SubInst, Sub, inst, regs, reg_gener)
            }
            // 通过类型转换，可以做到: FAdd 的输入一定是 Float 类型的寄存器
            middle::ir::instruction::InstType::FSub => {
                // ssa2tac_binary_float!(inst, regs, reg_gener, FSub, Sub, SubInst)
                ssa2tac_three_float!(SubInst, FSub, inst, regs, reg_gener, fmms)
            }
            middle::ir::instruction::InstType::Mul => {
                ssa2tac_three_usual_Rtype!(MulInst, Mul, inst, regs, reg_gener)
            }
            middle::ir::instruction::InstType::FMul => {
                // ssa2tac_binary_float!(inst, regs, reg_gener, FMul, Mul, MulInst)
                ssa2tac_three_float!(MulInst, FMul, inst, regs, reg_gener, fmms)
            }
            middle::ir::instruction::InstType::SDiv => {
                ssa2tac_three_usual_Rtype!(DivInst, SDiv, inst, regs, reg_gener)
            }
            middle::ir::instruction::InstType::SRem => {
                ssa2tac_three_usual_Rtype!(RemInst, SRem, inst, regs, reg_gener)
            }
            middle::ir::instruction::InstType::UDiv => todo!(), // TODO 目前还没有 udiv 和 urem
            middle::ir::instruction::InstType::URem => todo!(),
            middle::ir::instruction::InstType::FDiv => {
                ssa2tac_three_float!(DivInst, FDiv, inst, regs, reg_gener, fmms)
            }
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
                Self::build_gep_inst(gep, reg_gener, regs, stack_slots)
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
                let src = Self::no_load_from(itofp.get_src(), regs).with_context(|| context!())?;
                let dst = reg_gener.gen_virtual_float_reg();
                let fcvtsw = I2fInst::new(dst.into(), src);
                regs.insert(itofp as *const _ as Address, dst);
                Ok(vec![fcvtsw.into()])
            }
            middle::ir::instruction::InstType::FpToI => {
                let fptoi = downcast_ref::<middle::ir::instruction::extend_inst::FpToI>(
                    inst.as_ref().as_ref(),
                );
                let src = Self::no_load_from(fptoi.get_src(), regs).with_context(|| context!())?;
                let dst = reg_gener.gen_virtual_usual_reg();
                let fcvtws = F2iInst::new(dst.into(), src);
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
                // Self::build_call_inst(call, stack_allocator, stack_slots, reg_gener, regs)
                Self::build_call_inst(call, reg_gener, regs, fmms)
            }
        }
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
                let src =
                    Self::local_var_except_param_from(instr, regs).with_context(|| context!())?;
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
            middle::ir::ValueType::Int
            | middle::ir::ValueType::Float
            | middle::ir::ValueType::Bool
            | middle::ir::ValueType::SignedChar
            | middle::ir::ValueType::Pointer(_) => 8,
            middle::ir::ValueType::Void => {
                return Err(anyhow!("it can't alloca void")).with_context(|| context!())
            }
            middle::ir::ValueType::Array(_, _) => {
                let cap = Self::_cal_capas_factor(&ty).with_context(|| context!())?;
                // let
                // let dims =
                // let e_ty = Self::
                // let capa = Self::_cal_capas_rev(&ty);
                // let sz: usize = capa.iter().product();
                // (sz << 2/* *4 */) as u32
                // todo!()
                (cap << 2) as u32
            }
        };
        let ss = stack_allocator.alloc(bytes);
        stack_slots.insert(
            alloca as *const _ as Address, /* alloca 的目的寄存器, 里面存放有栈上变量的地址 */
            ss,                            /* 栈上分配的地址 */
        ); /* 将 栈上地址 与 目的寄存器 关联起来 */
        Ok(vec![])
    }

    /// store 指令，有几种可能: 指针/数组、全局变量、栈
    /// 要存的数据可能是: fmm/reg/imm
    pub fn build_store_inst(
        store: &middle::ir::instruction::memory_op_inst::Store,
        stack_slots: &HashMap<Address, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &HashMap<Address, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
    ) -> Result<Vec<Inst>> {
        let mut ret: Vec<Inst> = Vec::new();
        let addr =
            Self::address_from(store.get_ptr(), regs, stack_slots).with_context(|| context!())?;
        let (val, prepare) = Self::prepare_store_rs2(store.get_value(), reg_gener, regs, fmms)
            .with_context(|| context!())?;
        let Operand::Reg(val) = val else {
            // 注意, 这里可能会出现 fmm 的情况
            return Err(anyhow!("store value is not reg")).with_context(|| context!());
        };
        ret.extend(prepare);
        match addr {
            Operand::Reg(base) => {
                let sw = SwInst::new(val, 0.into(), base);
                ret.push(sw.into());
            }
            Operand::StackSlot(slot) => {
                let sd = StoreInst::new(slot, val);
                ret.push(sd.into());
            }
            Operand::Label(label) => {
                let addr = reg_gener.gen_virtual_usual_reg();
                let lla = LlaInst::new(addr, label);
                ret.push(lla.into());
                let sw = SwInst::new(val, 0.into(), addr);
                ret.push(sw.into());
            }
            _ => {
                return Err(anyhow!("impossible to store from imm/fmm"))
                    .with_context(|| context!());
            }
        }
        Ok(ret)
    }

    pub fn build_load_inst(
        load: &middle::ir::instruction::memory_op_inst::Load,
        stack_slots: &HashMap<Address, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
    ) -> Result<Vec<Inst>> {
        // dbg!(load);
        let mut ret: Vec<Inst> = Vec::new();
        // if regs.contains_key(&(load as *const _ as Address)) {
        //     unimplemented!() // 已经 load 过一次了
        // }
        let dst_reg = match load.get_value_type() {
            middle::ir::ValueType::Float => reg_gener.gen_virtual_float_reg(),
            middle::ir::ValueType::Int
            | middle::ir::ValueType::Bool
            | middle::ir::ValueType::Pointer(_) => reg_gener.gen_virtual_usual_reg(),
            _ => {
                return Err(anyhow!("load instruction to array/void")).with_context(|| context!());
            }
        };
        regs.insert(load as *const _ as Address, dst_reg);
        // 两种情况: 1. 从栈上获取(之前 alloca 过一次), 2. 从非栈上获取(parameter-pointer, global)
        let addr =
            Self::address_from(load.get_ptr(), regs, stack_slots).with_context(|| context!())?;
        match addr {
            Operand::Reg(base) => {
                let lw = LwInst::new(dst_reg, 0.into(), base);
                ret.push(lw.into());
            }
            Operand::StackSlot(slot) => {
                let ld = LoadInst::new(dst_reg, slot); // 对于 stack, 就是使用的 ld
                ret.push(ld.into());
            }
            Operand::Label(label) => {
                let addr = reg_gener.gen_virtual_usual_reg();
                let lla = LlaInst::new(addr, label);
                ret.push(lla.into());
                let lw = LwInst::new(dst_reg, 0.into(), addr);
                ret.push(lw.into());
            }
            _ => {
                return Err(anyhow!("impossible to load from imm/fmm")).with_context(|| context!());
            } // imm, fmm
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
                            let name = Self::fmm_lit_label_from(&fmm);
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
                    middle::ir::Constant::Array(_) | middle::ir::Constant::Zero(_) => {
                        return Err(anyhow!("return array is not allow:{}", op))
                            .with_context(|| context!())
                    }
                },
                middle::ir::Operand::Instruction(instr) => {
                    // let addr = instr.as_ref().as_ref() as *const dyn middle::ir::Instruction
                    //     as *const () as Address;
                    // let reg = regs
                    //     .get(&addr)
                    //     .ok_or(anyhow!("could not get {} from map", &addr).context(context!()))?; // 获取返回值对应的虚拟寄存器
                    let reg = Self::local_var_except_param_from(instr, regs)
                        .with_context(|| context!())?;
                    let mv_inst = match instr.get_value_type() {
                        middle::ir::ValueType::Int
                        | middle::ir::ValueType::Bool
                        | middle::ir::ValueType::SignedChar => {
                            MvInst::new(REG_A0.into(), reg.into())
                        }
                        middle::ir::ValueType::Float => MvInst::new(REG_FA0.into(), reg.into()),
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

    /// 既包含: 条件跳转, 也包含: 无条件跳转
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
}
