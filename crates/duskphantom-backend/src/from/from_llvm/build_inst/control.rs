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

// build control flow inst
use super::*;
use builder::IRBuilder;
use common::asm_of_insts;
impl IRBuilder {
    pub fn build_call_inst(
        call: &llvm_ir::instruction::Call,
        stack_slots: &mut HashMap<Name, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>
    ) -> Result<Vec<Inst>> {
        dbg!(call);
        dbg!(call.to_string());
        let f_name = match &call.function {
            rayon::iter::Either::Left(_) => todo!(),
            rayon::iter::Either::Right(op) => {
                Self::func_name_from(op).with_context(|| context!())?
            }
        };
        if f_name == "llvm.memset.p0.i64" {
            return Self::build_memset_inst(call, stack_slots, regs);
        }
        if f_name == "llvm.smax.i32" {
            return Self::build_call_smax_i32(call, reg_gener, regs);
        }

        let mut ret: Vec<Inst> = Vec::new();

        let mut i_arg: u32 = 0;
        let mut f_arg: u32 = 0;
        let mut extra_arg_stack: i64 = 0;
        let mut phisic_arg_regs: Vec<Reg> = Vec::new();
        for (arg, _) in &call.arguments {
            if let Ok(r) = Self::reg_from(arg, regs) {
                let r: Reg = r.try_into()?;
                if r.is_usual() && i_arg < 8 {
                    let reg = Reg::new(REG_A0.id() + i_arg, true);
                    phisic_arg_regs.push(reg);
                    let mv = MvInst::new(reg.into(), r.into());
                    ret.push(mv.into());
                    i_arg += 1;
                } else if !r.is_usual() && f_arg < 8 {
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
                    }
                } else if let Some(label) = v.label() {
                    let dst: Operand = if i_arg < 8 {
                        let reg = Reg::new(REG_A0.id() + i_arg, true);
                        phisic_arg_regs.push(reg);
                        i_arg += 1;
                        reg.into()
                    } else {
                        let off = extra_arg_stack.into();
                        extra_arg_stack += 8;
                        off
                    };
                    match dst {
                        Operand::Reg(reg) => {
                            let lla = LlaInst::new(reg, label);
                            ret.push(lla.into());
                        }
                        Operand::Imm(off) => {
                            let r = reg_gener.gen_virtual_usual_reg();
                            let lla = LlaInst::new(r, label);
                            ret.push(lla.into());
                            let sd = SdInst::new(r, off, REG_SP);
                            ret.push(sd.into());
                        }
                        _ => unimplemented!(),
                    }
                } else {
                    dbg!(v);
                    unimplemented!();
                }
            } else if let Ok(ss) = Self::stack_slot_from(arg, stack_slots) {
                // this case if for pass ptr of local var to function
                if i_arg < 8 {
                    let reg = Reg::new(REG_A0.id() + i_arg, true);
                    phisic_arg_regs.push(reg);
                    let laddr = LocalAddr::new(reg, ss);
                    ret.push(laddr.into());
                    i_arg += 1;
                } else {
                    let reg = reg_gener.gen_virtual_usual_reg();
                    let laddr = LocalAddr::new(reg, ss);
                    ret.push(laddr.into());

                    let sd = SdInst::new(reg, extra_arg_stack.into(), REG_SP);
                    ret.push(sd.into());
                    extra_arg_stack += 8;
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
                llvm_ir::Type::FuncType { result_type, param_types: _, is_var_arg: _ } => {
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

    /// build memset inst
    /// llvm ir 中 llvm.memset.p0.i64 函数的调用有四个参数
    /// arg0: dst ptr
    /// arg1: value
    /// arg2: len
    /// arg3: flag to control if use volatile (ignored in this IRBuilder)
    fn build_memset_inst(
        call: &llvm_ir::instruction::Call,
        stack_slots: &mut HashMap<Name, StackSlot>,
        regs: &mut HashMap<Name, Reg>
    ) -> Result<Vec<Inst>> {
        let f_name = match &call.function {
            rayon::iter::Either::Left(_) => todo!(),
            rayon::iter::Either::Right(op) => {
                Self::func_name_from(op).with_context(|| context!())?
            }
        };
        assert!(f_name == "llvm.memset.p0.i64");
        let f_name = "memset".to_string();
        let mut ret: Vec<Inst> = Vec::new();
        assert!(call.arguments.len() == 4);

        let mut phisic_arg_regs: Vec<Reg> = Vec::new();
        for (i_arg, (arg, _)) in call.arguments.iter().enumerate().take(3) {
            let i_arg = i_arg as u32;
            if let Ok(r) = Self::reg_from(arg, regs) {
                let r: Reg = r.try_into()?;
                assert!(r.is_usual());
                let arg = Reg::new(REG_A0.id() + i_arg, true);
                phisic_arg_regs.push(arg);
                let mv = MvInst::new(arg.into(), r.into());
                ret.push(mv.into());
            } else if let Ok(v) = Self::const_from(arg) {
                if let Some(imm) = v.imm() {
                    let reg = Reg::new(REG_A0.id() + i_arg, true);
                    phisic_arg_regs.push(reg);
                    let li = LiInst::new(reg.into(), imm.into());
                    ret.push(li.into());
                } else {
                    dbg!(v);
                    unimplemented!();
                }
            } else if let Ok(ss) = Self::stack_slot_from(arg, stack_slots) {
                // this case if for pass ptr of local var to function
                let reg = Reg::new(REG_A0.id() + i_arg, true);
                phisic_arg_regs.push(reg);
                let laddr = LocalAddr::new(reg, ss);
                ret.push(laddr.into());
            } else {
                dbg!(arg);
                unimplemented!();
            }
        }

        let mut call_inst = CallInst::new(f_name.to_string().into());
        call_inst.add_uses(&phisic_arg_regs); // set reg uses for call_inst

        call_inst.add_def(REG_A0);

        assert!(call.dest.is_none());
        ret.push(call_inst.into());

        Ok(ret)
    }

    /// build smax.i32 inst
    fn build_call_smax_i32(
        call: &llvm_ir::instruction::Call,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>
    ) -> Result<Vec<Inst>> {
        let f_name = match &call.function {
            rayon::iter::Either::Left(_) => todo!(),
            rayon::iter::Either::Right(op) => {
                Self::func_name_from(op).with_context(|| context!())?
            }
        };
        assert!(f_name == "llvm.smax.i32");
        let mut ret: Vec<Inst> = Vec::new();
        // choose the max value of two i32
        assert!(call.arguments.len() == 2);
        let op0 = &call.arguments[0].0;
        let op1 = &call.arguments[1].0;

        let (op0, pre_insert) = Self::prepare_usual_lhs(op0, reg_gener, regs)?;
        ret.extend(pre_insert);
        let (op1, pre_insert) = Self::prepare_usual_lhs(op1, reg_gener, regs)?;
        ret.extend(pre_insert);

        // if op0 > op1, ge_flag = 1, else ge_flag = 0
        let ge_flag = reg_gener.gen_virtual_usual_reg();
        let sltu = SltInst::new(ge_flag.into(), op1.clone(), op0.clone());
        ret.push(sltu.into());

        let (and_op, pre_insert) = Self::build_and_op(&ge_flag.into(), reg_gener)?;
        ret.extend(pre_insert);

        let and_op1 = reg_gener.gen_virtual_usual_reg();
        let not = NotInst::new(and_op1.into(), and_op.clone());
        ret.push(not.into());

        let add0 = reg_gener.gen_virtual_usual_reg();
        let add = AddInst::new(add0.into(), and_op, op0.clone());
        ret.push(add.into());

        let add1 = reg_gener.gen_virtual_usual_reg();
        let add = AddInst::new(add1.into(), and_op1.into(), op1.clone());
        ret.push(add.into());

        let max = reg_gener.gen_virtual_usual_reg();
        let add = AddInst::new(max.into(), op0, and_op1.into());
        ret.push(add.into());

        regs.insert(call.dest.clone().unwrap(), max);

        dbg!(&ret);
        println!("{}", asm_of_insts(&ret));

        Ok(ret)
    }
}
