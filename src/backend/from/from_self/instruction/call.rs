pub use super::*;

impl IRBuilder {
    pub fn build_call_inst(
        call: &middle::ir::instruction::misc_inst::Call,
        // stack_allocator: &mut StackAllocator,
        // stack_slots: &HashMap<Address, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
    ) -> Result<Vec<Inst>> {
        if call.func.name.contains("llvm.memset") {
            return Self::build_memset_inst(call, regs);
        }

        let mut ret_insts: Vec<Inst> = Vec::new(); // build_call_inst 的返回值

        /* ---------- 参数 ---------- */

        let mut i_arg_num: u32 = 0;
        let mut f_arg_num: u32 = 0;
        let mut extra_arg_stack: i64 = 0;
        let mut phisic_arg_regs: Vec<Reg> = Vec::new();
        let arguments = call.get_operand(); // 参数列表, 这个可以类比成 llvm_ir::call::arguments
        for arg in arguments {
            let ope = Self::no_load_from(arg, regs).with_context(|| context!())?;
            match ope {
                Operand::Reg(r) => {
                    if r.is_usual() && i_arg_num < 8 {
                        // i reg
                        let reg = Reg::new(REG_A0.id() + i_arg_num, true);
                        phisic_arg_regs.push(reg);
                        let mv = MvInst::new(reg.into(), ope);
                        ret_insts.push(mv.into());
                        i_arg_num += 1;
                    } else if (!r.is_usual()) && f_arg_num < 8 {
                        // f reg
                        let reg = Reg::new(REG_FA0.id() + f_arg_num, false);
                        phisic_arg_regs.push(reg);
                        let mv = MvInst::new(reg.into(), ope);
                        ret_insts.push(mv.into());
                        f_arg_num += 1;
                    } else {
                        // 额外参数 reg
                        let sd = SdInst::new(r, extra_arg_stack.into(), REG_SP);
                        extra_arg_stack += 8;
                        ret_insts.push(sd.into());
                    }
                }
                Operand::Imm(imm) => {
                    if i_arg_num < 8 {
                        // imm
                        let reg = Reg::new(REG_A0.id() + i_arg_num, true);
                        let li = LiInst::new(reg.into(), imm.into());
                        phisic_arg_regs.push(reg);
                        ret_insts.push(li.into());
                        i_arg_num += 1;
                    } else {
                        // imm 额外参数
                        let reg = reg_gener.gen_virtual_usual_reg();
                        let li = LiInst::new(reg.into(), imm.into());
                        ret_insts.push(li.into());
                        let sd = SdInst::new(reg, extra_arg_stack.into(), REG_SP);
                        extra_arg_stack += 8;
                        ret_insts.push(sd.into());
                    }
                }
                Operand::Fmm(fmm) => {
                    if f_arg_num < 8 {
                        // fmm
                        let p_reg = Reg::new(REG_FA0.id() + f_arg_num, false);
                        phisic_arg_regs.push(p_reg);
                        let (v_reg, prepare) = Self::_prepare_fmm(&fmm, reg_gener, fmms)
                            .with_context(|| context!())?;
                        ret_insts.extend(prepare);
                        let mv = MvInst::new(p_reg.into(), v_reg.into());
                        ret_insts.push(mv.into());
                        f_arg_num += 1;
                    } else {
                        // fmm 额外参数
                        let (v_reg, prepare) = Self::_prepare_fmm(&fmm, reg_gener, fmms)
                            .with_context(|| context!())?;
                        ret_insts.extend(prepare);
                        let sd = SdInst::new(v_reg, extra_arg_stack.into(), REG_SP);
                        extra_arg_stack += 8;
                        ret_insts.push(sd.into());
                    }
                }
                /* TODO Operand::StackSlot(_) 可能是局部数组 */
                _ => {
                    /*  Operand::Label(_) */
                    return Err(anyhow!("argument can't be a label".to_string()))
                        .with_context(|| context!());
                }
            }
        }

        /* ---------- call 指令本身 ---------- */

        // 函数是全局的，因此用的是名字
        let mut call_inst: CallInst = CallInst::new(call.func.name.to_string().into()); // call <一个全局的 name >
        call_inst.add_uses(&phisic_arg_regs); // set reg uses for call_inst

        let call_addr = call as *const _ as Address;

        let func = call.func;

        /* ---------- 返回值 ---------- */

        // call 返回之后，将返回值放到一个虚拟寄存器中
        match func.return_type {
            middle::ir::ValueType::Void => {
                ret_insts.push(call_inst.into());
            }
            middle::ir::ValueType::Int
            | middle::ir::ValueType::Float
            | middle::ir::ValueType::Bool => {
                let (dst, ret_a0) = if func.return_type == middle::ir::ValueType::Int
                    || func.return_type == middle::ir::ValueType::Bool
                {
                    (reg_gener.gen_virtual_usual_reg(), REG_A0)
                } else {
                    (reg_gener.gen_virtual_float_reg(), REG_FA0)
                };
                let mv = MvInst::new(dst.into(), ret_a0.into());
                regs.insert(call_addr, dst); // 绑定中端的 id 和 虚拟寄存器
                call_inst.add_def(ret_a0);
                ret_insts.push(call_inst.into());
                ret_insts.push(mv.into());
            }
            _ => {
                return Err(anyhow!("sysy only return: void | float | int".to_string()))
                    .with_context(|| context!())
            }
        };

        Ok(ret_insts)
    }

    fn build_memset_inst(
        call: &middle::ir::instruction::misc_inst::Call,
        regs: &mut HashMap<Address, Reg>,
    ) -> Result<Vec<Inst>> {
        assert!(call.func.name.contains("llvm.memset"));
        let mut ret: Vec<Inst> = Vec::new();
        let args = call.get_operand(); // 实参
        assert!(args.len() == 4);
        let mut phisic_arg_regs: Vec<Reg> = Vec::new();
        for (i_arg, arg) in args.iter().enumerate().take(3) {
            let i_arg = i_arg as u32; // 第几个参数
            let ope = Self::no_load_from(arg, regs).with_context(|| context!())?;
            match ope {
                Operand::Reg(reg) => {
                    assert!(reg.is_usual());
                    let a_n = Reg::new(REG_A0.id() + i_arg, true);
                    phisic_arg_regs.push(a_n);
                    let mv = MvInst::new(a_n.into(), reg.into());
                    ret.push(mv.into());
                }
                Operand::Imm(imm) => {
                    let a_n = Reg::new(REG_A0.id() + i_arg, true);
                    phisic_arg_regs.push(a_n);
                    let li = LiInst::new(a_n.into(), imm.into());
                    ret.push(li.into());
                }
                _ => unimplemented!(), /* Operand::Fmm(_) Operand::StackSlot(_) Operand::Label(_) */
            }
        }
        let mut call_inst = CallInst::new("memset".to_string().into());
        call_inst.add_uses(&phisic_arg_regs); // set reg uses for call_inst
        call_inst.add_def(REG_A0);
        // assert!(call.dest.is_none());
        ret.push(call_inst.into());
        Ok(ret)
    }
}
