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

use super::*;
use anyhow::Result;
use std::collections::HashMap;

use super::Address;
use crate::backend::*;
use crate::middle;
use crate::utils::mem::ObjPtr;

pub struct IRBuilder;

impl IRBuilder {
    pub fn gen_from_self(program: &middle::Program) -> Result<Program> {
        let self_module = &program.module;
        // dbg!(&llvm.types);
        let mut global_vars = Self::build_global_var(&self_module.global_variables)?;
        let mut fmms: HashMap<Fmm, FloatVar> = HashMap::new();

        // dbg!(&global_vars);
        let funcs = Self::build_funcs(&self_module.functions, &mut fmms)?;

        for (_, float_var) in fmms {
            global_vars.push(float_var.into());
        }

        let mdl = module::Module {
            name: "main".to_string(),
            entry: Some("main".to_string()),
            global: global_vars,
            funcs,
        };

        Ok(prog::Program {
            entry: Some(mdl.name.clone()),
            modules: vec![mdl],
        })
    }

    pub fn build_funcs(
        self_funcs: &Vec<middle::ir::FunPtr>,
        fmms: &mut HashMap<Fmm, FloatVar>,
    ) -> Result<Vec<Func>> {
        let mut funcs = Vec::new();
        let mut caller_regs_stacks: HashMap<String, u32> = HashMap::new();
        for self_func in self_funcs {
            // Do not build library function
            if self_func.is_lib() {
                continue;
            }

            // Build the function
            let fu = self_func.as_ref();
            let (mut func, caller_regs_stack) = Self::build_func(fu, fmms)?;

            Self::label_rename_func(&mut func, fu)?;

            caller_regs_stacks.insert(func.name().to_string(), caller_regs_stack);
            funcs.push(func);
        }
        let max_callee_regs_stacks =
            Self::prepare_max_callee_regs_stack(&mut funcs, &caller_regs_stacks)?;
        max_callee_regs_stacks.iter().for_each(|(func, n)| {
            if let Some(f) = funcs.iter_mut().find(|f| f.name() == func) {
                f.max_callee_regs_stack = *n;
            }
        });
        Self::realloc_stack_slots(&mut funcs, &max_callee_regs_stacks)?;
        Ok(funcs)
    }

    pub fn build_func(
        self_func: &middle::ir::Function,
        fmms: &mut HashMap<Fmm, FloatVar>,
    ) -> Result<(Func, u32)> {
        /* ---------- 初始化一些分配器 ---------- */
        let mut stack_allocator = StackAllocator::new();
        let mut stack_slots: HashMap<Address, StackSlot> = HashMap::new();
        let mut reg_gener = RegGenerator::new();
        let mut regs: HashMap<Address, Reg> = HashMap::new();
        let mut insert_back_for_remove_phi = HashMap::new();

        /* ---------- 根据 entry 创建 func ---------- */
        let (entry, caller_reg_stack) = Self::build_entry(
            self_func,
            &mut stack_allocator,
            &mut stack_slots,
            &mut reg_gener,
            &mut regs,
            fmms,
            &mut insert_back_for_remove_phi,
        )?;
        let params: Vec<_> = self_func.params.iter().map(|p| p.name.clone()).collect();
        let mut m_f = Func::new(self_func.name.clone(), params, entry);
        // *m_f.caller_regs_stack_mut() = Some(caller_reg_stack.try_into()?); // caller_reg_stack 是 build_entry 的时候确定的, 然后绑定到函数里面

        /* ---------- 返回值 ---------- */
        match &self_func.return_type {
            middle::ir::ValueType::Void => { /* do nothing */ }
            middle::ir::ValueType::Int
            | middle::ir::ValueType::Bool
            | middle::ir::ValueType::Pointer(_) => {
                m_f.ret_mut().replace(REG_A0);
            }
            middle::ir::ValueType::Float => {
                m_f.ret_mut().replace(REG_FA0);
            }
            middle::ir::ValueType::Array(_, _) => todo!(),
            _ => todo!(),
        }

        /* ---------- build other bbs ---------- */
        for bb in Self::build_other_bbs(
            self_func,
            &mut stack_allocator,
            &mut stack_slots,
            &mut reg_gener,
            &mut regs,
            fmms,
            &mut insert_back_for_remove_phi,
        )? {
            m_f.push_bb(bb);
        }

        /* ---------- phi ---------- */
        let mut bbs_mut = m_f // insert back to bbs to process phi
            .iter_bbs_mut()
            .map(|bb| (bb.label().to_string(), bb))
            .collect::<HashMap<String, &mut Block>>();
        for (bb_name, insert_back) in insert_back_for_remove_phi {
            let bb = bbs_mut
                .get_mut(&bb_name)
                .ok_or_else(|| anyhow!("{:?} not found", &&bb_name))
                .with_context(|| context!())?;
            for (from, phi_dst) in insert_back {
                let from = Self::no_load_from(&from, &regs)?;
                match from {
                    Operand::Reg(_) => {
                        let mv = MvInst::new(phi_dst.into(), from);
                        bb.insert_before_term(mv.into())?;
                    }
                    Operand::Imm(_) => {
                        let li = LiInst::new(phi_dst.into(), from);
                        bb.insert_before_term(li.into())?;
                    }
                    Operand::Fmm(fmm) => {
                        let lit = if let Some(f_var) = fmms.get(&fmm) {
                            f_var.name.clone()
                        } else {
                            let name = Self::fmm_lit_label_from(&fmm);
                            fmms.insert(
                                fmm.clone(),
                                FloatVar {
                                    name: name.clone(),
                                    init: Some(fmm.clone().try_into()?),
                                    is_const: true,
                                },
                            );
                            name
                        };
                        let addr = reg_gener.gen_virtual_usual_reg();
                        let lla = LlaInst::new(addr, lit.into());
                        bb.insert_before_term(lla.into())?;
                        let loadf = LwInst::new(phi_dst, 0.into(), addr);
                        bb.insert_before_term(loadf.into())?;
                    }
                    _ => return Err(anyhow!("not support {:?}", from)), /* stackslot(_), label(_) */
                }
            }
        }

        /* ---------- stack allocator, 后面物理化也会用到 ---------- */
        *m_f.stack_allocator_mut() = Some(stack_allocator);
        *m_f.reg_gener_mut() = Some(reg_gener);

        Ok((m_f, caller_reg_stack.try_into()?))
    }

    /// caller_regs_stack 是在 build 单个 func 的时候确定的
    /// 这里一定要放在 build_funcs 之后, 因为这个时候，所有的函数的 caller_regs_stack 才会被计算好
    fn prepare_max_callee_regs_stack(
        funcs: &mut Vec<Func>,
        caller_regs_stacks: &HashMap<String, u32>,
    ) -> Result<HashMap<String, u32>> {
        let mut max_callee_regs_stacks: HashMap<String, u32> = HashMap::new();
        for f in funcs {
            let mut max_callee_regs_stack = 0;
            for bb in f.iter_bbs() {
                for inst in bb.insts() {
                    if let Inst::Call(c) = inst {
                        let callee_regs_stack =
                            *caller_regs_stacks.get(c.func_name().as_str()).unwrap_or(&0);
                        max_callee_regs_stack =
                            std::cmp::max(max_callee_regs_stack, callee_regs_stack);
                    }
                }
                max_callee_regs_stacks.insert(f.name().to_string(), max_callee_regs_stack);
            }
        }
        Ok(max_callee_regs_stacks)
    }

    fn realloc_stack_slots(
        funcs: &mut Vec<Func>,
        max_callee_regs_stacks: &HashMap<String, u32>,
    ) -> Result<()> {
        for f in funcs {
            let mut new_stack_allocator = StackAllocator::new();

            /* ---------- 为额外参数分配空间 ---------- */

            let max_callee_regs_need = *max_callee_regs_stacks.get(f.name()).unwrap_or(&0);
            new_stack_allocator.alloc(max_callee_regs_need);

            /* ---------- 重排序栈 ---------- */

            let mut old_stack_slots: HashMap<StackSlot, usize /* times */> = HashMap::new();

            /* ***** 统计每个 slot 使用的次数 ***** */
            for bb in f.iter_bbs() {
                for inst in bb.insts() {
                    let Some(stack_slot) = inst.stack_slot().cloned() else {
                        continue;
                    };
                    let new_times = old_stack_slots.get(&stack_slot).unwrap_or(&0) + 1;
                    old_stack_slots.insert(stack_slot, new_times);
                }
            }

            /* ***** 按照 times 排序 ***** */
            let mut old_stack_slots: Vec<(StackSlot, usize)> =
                old_stack_slots.into_iter().collect();
            old_stack_slots.sort_by(|a, b| a.1.cmp(&b.1)); // 按照 times 排序

            /* ***** 分配到两边 ***** */
            let ordered_stack_slots = {
                let mut left_sss: Vec<StackSlot> = Vec::new();
                let mut right_sss: Vec<StackSlot> = Vec::new();
                for (idx /* 下标, 奇偶 */, (ss, _ /* times 用不到了 */)) in
                    old_stack_slots.iter().rev().enumerate()
                {
                    if idx % 2 == 0 {
                        left_sss.push(*ss);
                    } else {
                        right_sss.push(*ss);
                    }
                }
                left_sss.extend(right_sss.iter().rev());
                left_sss
            };

            /* ***** 构造一个新的 allocator ***** */
            let new_stack_slots: HashMap<StackSlot, StackSlot> = ordered_stack_slots
                .iter()
                .map(|&ss| (ss, new_stack_allocator.alloc(ss.size())))
                .collect();

            for bb in f.iter_bbs_mut() {
                for inst in bb.insts_mut() {
                    match inst {
                        Inst::Load(load) => {
                            let new_ss = new_stack_slots
                                .get(load.src())
                                .ok_or_else(|| {
                                    anyhow!("not found mapping of stack slot {:?}", load.src())
                                })
                                .with_context(|| context!())?;
                            *load.src_mut() = *new_ss;
                        }
                        Inst::Store(store) => {
                            let new_ss = new_stack_slots
                                .get(store.dst())
                                .ok_or_else(|| {
                                    anyhow!("not found mapping of stack slot {:?}", store.src())
                                })
                                .with_context(|| context!())?;
                            *store.dst_mut() = *new_ss;
                        }
                        Inst::LocalAddr(local_addr) => {
                            let new_ss = new_stack_slots
                                .get(local_addr.stack_slot())
                                .ok_or_else(|| {
                                    anyhow!(
                                        "not found mapping of stack slot {:?}",
                                        local_addr.stack_slot()
                                    )
                                })
                                .with_context(|| context!())?;
                            *local_addr.stack_slot_mut() = *new_ss;
                        }
                        _ => {
                            continue;
                        }
                    }
                }
            }

            f.stack_allocator_mut().replace(new_stack_allocator);
        }
        Ok(())
    }

    fn build_other_bbs(
        func: &middle::ir::Function,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Address, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
        insert_back_for_remove_phi: &mut HashMap<String, Vec<(middle::ir::Operand, Reg)>>,
    ) -> Result<Vec<Block>> {
        func.bfs_iter()
            .skip(1)
            .map(|ptr_bb| {
                Self::build_bb(
                    &ptr_bb,
                    stack_allocator,
                    stack_slots,
                    reg_gener,
                    regs,
                    fmms,
                    insert_back_for_remove_phi,
                )
            })
            .collect()
    }

    fn build_bb(
        bb: &ObjPtr<middle::ir::BasicBlock>,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Address, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
        insert_back_for_remove_phi: &mut HashMap<String, Vec<(middle::ir::Operand, Reg)>>,
    ) -> Result<Block> {
        // basic 的 label 注意一下
        let mut m_bb = Block::new(Self::label_name_from(bb));
        for inst in bb.iter() {
            let gen_insts = Self::build_instruction(
                &inst,
                stack_allocator,
                stack_slots,
                reg_gener,
                regs,
                fmms,
                insert_back_for_remove_phi,
            )
            .with_context(|| context!())?;
            m_bb.extend_insts(gen_insts);
        }
        m_bb.depth = bb.depth;
        Ok(m_bb)
    }

    fn build_entry(
        func: &middle::ir::Function,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Address, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
        insert_back_for_remove_phi: &mut HashMap<String, Vec<(middle::ir::Operand, Reg)>>,
    ) -> Result<(Block, usize)> {
        /* ---------- 初始化 ---------- */
        let mut insts: Vec<Inst> = Vec::new();

        /* ---------- 函数形参 ---------- */
        let mut caller_regs_stack = 0;
        let mut float_idx = 0;
        let mut usual_idx = 0;
        for param in func.params.iter() {
            let is_usual: bool = match &param.value_type {
                middle::ir::ValueType::Float => false,
                middle::ir::ValueType::Pointer(_)
                | middle::ir::ValueType::Bool
                | middle::ir::ValueType::Int => true,
                middle::ir::ValueType::Void => {
                    return Err(anyhow!(
                        "it is impossible to receive void-type parameter: {}",
                        param
                    ))
                }
                middle::ir::ValueType::Array(_, _) => {
                    return Err(anyhow!("array should be pointer {}", param))
                }
                middle::ir::ValueType::SignedChar => todo!(),
            };
            let v_reg = reg_gener.gen_virtual_reg(is_usual);
            regs.insert(param.as_ref() as *const _ as Address, v_reg); // 参数绑定寄存器
            if is_usual && usual_idx <= 7 {
                let a_reg = Reg::new(REG_A0.id() + usual_idx, is_usual);
                let mv = MvInst::new(v_reg.into(), a_reg.into());
                insts.push(mv.into());
                usual_idx += 1;
            } else if !is_usual && float_idx <= 7 {
                let a_reg = Reg::new(REG_FA0.id() + float_idx, is_usual);
                let mv = MvInst::new(v_reg.into(), a_reg.into());
                insts.push(mv.into());
                float_idx += 1;
            } else if (is_usual && usual_idx > 7) || (!is_usual && float_idx > 7) {
                let ld_inst = LdInst::new(v_reg, caller_regs_stack.into(), REG_S0);
                insts.push(ld_inst.into());
                caller_regs_stack += 8;
            }
        }

        /* ---------- 指令选择 ---------- */
        let bb = func.entry.with_context(|| context!())?;
        for inst in bb.iter() {
            let gen_insts = Self::build_instruction(
                &inst,
                stack_allocator,
                stack_slots,
                reg_gener,
                regs,
                fmms,
                insert_back_for_remove_phi,
            )
            .with_context(|| context!())?;
            insts.extend(gen_insts);
        }

        /* ---------- 后端的 entry bb ---------- */
        let label = Self::label_name_from(&bb);
        let mut entry = Block::new(label);
        entry.extend_insts(insts);
        // 设置循环深度
        entry.depth = bb.depth;

        let caller_regs_stack = usize::try_from(caller_regs_stack)?;
        Ok((entry, caller_regs_stack))
    }
}
