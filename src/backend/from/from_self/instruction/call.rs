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
        let mut call_insts: Vec<Inst> = Vec::new(); // build_call_inst 的返回值

        /* ---------- 参数 ---------- */

        let mut i_arg_num: u32 = 0;
        let mut f_arg_num: u32 = 0;
        let mut extra_arg_stack: i64 = 0;
        let mut phisic_arg_regs: Vec<Reg> = Vec::new();
        let arguments = call.get_operand(); // 参数列表, 这个可以类比成 llvm_ir::call::arguments
        for arg in arguments {
            let ope = Self::no_load_from(arg, regs).context(context!())?;
            match ope {
                Operand::Reg(r) => {
                    if r.is_usual() && i_arg_num < 8 {
                        // i reg
                        let reg = Reg::new(REG_A0.id() + i_arg_num, true);
                        phisic_arg_regs.push(reg);
                        let mv = MvInst::new(reg.into(), ope);
                        call_insts.push(mv.into());
                        i_arg_num += 1;
                    } else if (!r.is_usual()) && f_arg_num < 8 {
                        // f reg
                        let reg = Reg::new(REG_FA0.id() + f_arg_num, false);
                        phisic_arg_regs.push(reg);
                        let mv = MvInst::new(reg.into(), ope);
                        call_insts.push(mv.into());
                        f_arg_num += 1;
                    } else {
                        // 额外参数 reg
                        let sd = SdInst::new(r, extra_arg_stack.into(), REG_SP);
                        extra_arg_stack += 8;
                        call_insts.push(sd.into());
                    }
                }
                Operand::Imm(imm) => {
                    if i_arg_num < 8 {
                        // imm
                        let reg = Reg::new(REG_A0.id() + i_arg_num, true);
                        let li = LiInst::new(reg.into(), imm.into());
                        phisic_arg_regs.push(reg);
                        call_insts.push(li.into());
                        i_arg_num += 1;
                    } else {
                        // imm 额外参数
                        let reg = reg_gener.gen_virtual_usual_reg();
                        let li = LiInst::new(reg.into(), imm.into());
                        call_insts.push(li.into());
                        let sd = SdInst::new(reg, extra_arg_stack.into(), REG_SP);
                        extra_arg_stack += 8;
                        call_insts.push(sd.into());
                    }
                }
                Operand::Fmm(fmm) => {
                    if f_arg_num < 8 {
                        // fmm
                        let p_reg = Reg::new(REG_FA0.id() + f_arg_num, false);
                        phisic_arg_regs.push(p_reg);
                        let (v_reg, prepare) = Self::_prepare_fmm(&fmm, reg_gener, fmms)
                            .with_context(|| context!())?;
                        call_insts.extend(prepare);
                        let mv = MvInst::new(p_reg.into(), v_reg.into());
                        call_insts.push(mv.into());
                        f_arg_num += 1;
                    } else {
                        // fmm 额外参数
                        let (v_reg, prepare) = Self::_prepare_fmm(&fmm, reg_gener, fmms)
                            .with_context(|| context!())?;
                        call_insts.extend(prepare);
                        let sd = SdInst::new(v_reg, extra_arg_stack.into(), REG_SP);
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
        call_inst.add_uses(&phisic_arg_regs); // set reg uses for call_inst

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
