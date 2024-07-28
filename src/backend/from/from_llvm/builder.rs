use super::super::prog::Program;
use super::*;

use anyhow::{Context, Result};

use llvm_ir::Name;
use std::collections::HashMap;
use var::FloatVar;

pub struct IRBuilder;

impl IRBuilder {
    #[cfg(feature = "clang_enabled")]
    #[allow(unused)]
    pub fn gen_from_clang(program: &clang_frontend::Program) -> Result<Program> {
        let llvm_module = &program.llvm;
        let mut global_vars = Self::build_global_var(&llvm_module.global_vars)?;
        let mut fmms: HashMap<Fmm, FloatVar> = HashMap::new();
        let funcs = Self::build_funcs(&llvm_module.functions, &mut fmms)?;
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
            entry: None,
            modules: vec![mdl],
        })
    }

    /**
     * build funcs
     */
    pub fn build_funcs(
        llvm_funcs: &[llvm_ir::Function],
        fmms: &mut HashMap<Fmm, FloatVar>,
    ) -> Result<Vec<Func>> {
        let mut funcs = Vec::new();
        let mut caller_regs_stacks: HashMap<String, u32> = HashMap::new();
        for llvm_func in llvm_funcs {
            let (func, caller_regs_stack) = Self::build_func(llvm_func, fmms)?;
            caller_regs_stacks.insert(func.name().to_string(), caller_regs_stack);
            funcs.push(func);
        }
        // count max_callee_regs_stack
        let max_callee_regs_stacks =
            Self::prepare_max_callee_regs_stack(&mut funcs, &caller_regs_stacks)?;
        // realloc stack slots considering max_callee_regs_stack
        Self::realloc_stack_slots(&mut funcs, &max_callee_regs_stacks)?;

        Ok(funcs)
    }

    /// build on-building func from llvm_ir::Function
    /// # Arguments
    /// * `llvm_func` - llvm_ir::Function
    /// * `fmms` - HashMap<Fmm, FloatVar>
    /// # Returns
    /// `Result<(f:Func,caller_regs_stack:u32)>`
    /// - f: the function, which is still on building,not condsidering the max_callee_regs_stack yet
    /// - caller_regs_stack: the number of regs that the caller function needs to save
    pub fn build_func(
        llvm_func: &llvm_ir::Function,
        fmms: &mut HashMap<Fmm, FloatVar>,
    ) -> Result<(Func, u32)> {
        let args: Vec<String> = llvm_func
            .parameters
            .iter()
            .map(|p| p.name.to_string())
            .collect();
        let mut insert_back_for_remove_phi: HashMap<String, Vec<(llvm_ir::operand::Operand, Reg)>> =
            HashMap::new();
        let mut reg_gener = RegGenerator::new();
        let mut regs: HashMap<Name, Reg> = HashMap::new();
        let mut stack_allocator = StackAllocator::new();
        let mut stack_slots: HashMap<Name, StackSlot> = HashMap::new();

        let (entry, caller_reg_stack) = Self::build_entry(
            llvm_func,
            &mut stack_allocator,
            &mut stack_slots,
            &mut reg_gener,
            &mut regs,
            fmms,
            &mut insert_back_for_remove_phi,
        )?;
        let mut m_f = Func::new(llvm_func.name.to_string(), args, entry);

        let ret_ty = llvm_func.return_type.as_ref();
        if Self::is_ty_float(ret_ty) {
            m_f.ret_mut().replace(REG_FA0);
        } else if Self::is_ty_int(ret_ty) {
            m_f.ret_mut().replace(REG_A0);
        } else if Self::is_ty_void(ret_ty) {
            // do nothing
        } else {
            unimplemented!("return type is not int or float");
        }

        for bb in Self::build_other_bbs(
            llvm_func,
            &mut stack_allocator,
            &mut stack_slots,
            &mut reg_gener,
            &mut regs,
            fmms,
            &mut insert_back_for_remove_phi,
        )? {
            m_f.push_bb(bb);
        }

        // insert back to bbs to process phi
        let mut bbs_mut = m_f
            .iter_bbs_mut()
            .map(|bb| (bb.label().to_string(), bb))
            .collect::<HashMap<String, &mut Block>>();
        for (bb_name, insert_back) in insert_back_for_remove_phi {
            let bb = bbs_mut
                .get_mut(&bb_name)
                .ok_or_else(|| anyhow!("{:?} not found", &bb_name))
                .with_context(|| context!())?;
            for (from, phi_dst) in insert_back {
                let from = Self::value_from(&from, &regs)?;
                match from {
                    Operand::Reg(_) => {
                        let mv = MvInst::new(phi_dst.into(), from);
                        bb.insert_before_term(mv.into())?;
                    }
                    Operand::Imm(_) => {
                        let li = LiInst::new(phi_dst.into(), from);
                        bb.insert_before_term(li.into())?;
                    }
                    _ => unimplemented!(),
                }
            }
        }

        *m_f.stack_allocator_mut() = Some(stack_allocator);
        Ok((m_f, caller_reg_stack.try_into()?))
    }

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
            }
            max_callee_regs_stacks.insert(f.name().to_string(), max_callee_regs_stack);
        }
        Ok(max_callee_regs_stacks)
    }

    fn realloc_stack_slots(
        funcs: &mut Vec<Func>,
        max_callee_regs_stacks: &HashMap<String, u32>,
    ) -> Result<()> {
        for f in funcs {
            let mut old_stack_slots: HashMap<StackSlot, usize> = HashMap::new();
            let mut new_stack_allocator = StackAllocator::new();
            for bb in f.iter_bbs() {
                for inst in bb.insts() {
                    let stack_slot = match inst {
                        Inst::Load(load) => *load.src(),
                        Inst::Store(store) => *store.dst(),
                        _ => {
                            continue;
                        }
                    };
                    let new_times = old_stack_slots.get(&stack_slot).unwrap_or(&0) + 1;
                    old_stack_slots.insert(stack_slot, new_times);
                }
            }
            let max_callee_regs_need = *max_callee_regs_stacks.get(f.name()).unwrap_or(&0);
            new_stack_allocator.alloc(max_callee_regs_need);
            let mut old_stack_slots: Vec<(StackSlot, usize)> =
                old_stack_slots.into_iter().collect();
            old_stack_slots.sort_by(|a, b| a.1.cmp(&b.1));

            let order_stack_slots = |old_stack_slots: Vec<(StackSlot, usize)>| {
                let mut left_sss: Vec<StackSlot> = Vec::new();
                let mut right_sss: Vec<StackSlot> = Vec::new();
                for (idx, (ss, _)) in old_stack_slots.iter().rev().enumerate() {
                    if idx % 2 == 0 {
                        left_sss.push(*ss);
                    } else {
                        right_sss.push(*ss);
                    }
                }
                left_sss.extend(right_sss.iter().rev());
                left_sss
            };
            let ordered_stack_slots = order_stack_slots(old_stack_slots);

            let mut new_stack_slots: HashMap<StackSlot, StackSlot> = HashMap::new();
            for ss in ordered_stack_slots {
                let new_ss = new_stack_allocator.alloc(ss.size());
                new_stack_slots.insert(ss, new_ss);
            }

            for bb in f.iter_bbs_mut() {
                for inst in bb.insts_mut() {
                    match inst {
                        Inst::Load(load) => {
                            let new_ss = new_stack_slots
                                .get(load.src())
                                .ok_or_else(|| {
                                    anyhow!("not found mapping of stack slot {:?} ", load.src())
                                })
                                .with_context(|| context!())?;
                            *load.src_mut() = *new_ss;
                        }
                        Inst::Store(store) => {
                            let new_ss = new_stack_slots
                                .get(store.dst())
                                .ok_or_else(|| {
                                    anyhow!("not found mapping of stack slot {:?} ", store.dst())
                                })
                                .with_context(|| context!())?;
                            *store.dst_mut() = *new_ss;
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

    fn build_entry(
        f: &llvm_ir::Function,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Name, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
        insert_back_for_remove_phi: &mut HashMap<String, Vec<(llvm_ir::operand::Operand, Reg)>>,
    ) -> Result<(Block, usize)> {
        let bb = f
            .basic_blocks
            .first()
            .ok_or(anyhow!("no basic block"))
            .with_context(|| context!())?;
        let mut insts = Vec::new();
        let mut caller_regs_stack = 0;
        let mut float_idx = 0;
        let mut usual_idx = 0;
        for param in f.parameters.iter() {
            let is_usual = if Self::is_ty_int(&param.ty) {
                true
            } else {
                assert!(Self::is_ty_float(&param.ty));
                false
            };
            let v_reg = reg_gener.gen_virtual_reg(is_usual);
            regs.insert(param.name.clone(), v_reg);
            if is_usual && usual_idx <= 7 {
                let a_reg = Reg::new(REG_A0.id() + usual_idx, is_usual);
                let mv = MvInst::new(v_reg.into(), a_reg.into());
                insts.push(mv.into());
                usual_idx += 1;
            }
            if !is_usual && float_idx <= 7 {
                let a_reg = Reg::new(REG_FA0.id() + float_idx, is_usual);
                let mv = MvInst::new(v_reg.into(), a_reg.into());
                insts.push(mv.into());
                float_idx += 1;
            }
            if (is_usual && usual_idx > 7) || (!is_usual && float_idx > 7) {
                let ld_inst = LdInst::new(v_reg, caller_regs_stack.into(), REG_S0);
                insts.push(ld_inst.into());
                caller_regs_stack += 8;
            }
        }

        for inst in &bb.instrs {
            let gen_insts = Self::build_instruction(
                inst,
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

        insts.extend(Self::build_term_inst(&bb.term, reg_gener, regs, fmms)?);

        let mut entry = Block::new(Self::label_name_from(&bb.name).with_context(|| context!())?);
        entry.extend_insts(insts);
        let caller_regs_stack = usize::try_from(caller_regs_stack)?; // 这是将 i64 转换为 usize
        Ok((entry, caller_regs_stack))
    }

    fn build_other_bbs(
        f: &llvm_ir::Function,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Name, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
        insert_back_for_remove_phi: &mut HashMap<String, Vec<(llvm_ir::operand::Operand, Reg)>>,
    ) -> Result<Vec<Block>> {
        let mut ret: Vec<Block> = Vec::new();
        for bb in &f.basic_blocks[1..] {
            let m_bb = Self::build_bb(
                bb,
                stack_allocator,
                stack_slots,
                reg_gener,
                regs,
                fmms,
                insert_back_for_remove_phi,
            )?;
            ret.push(m_bb);
        }
        Ok(ret)
    }

    fn build_bb(
        bb: &llvm_ir::BasicBlock,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Name, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
        insert_back_for_remove_phi: &mut HashMap<String, Vec<(llvm_ir::operand::Operand, Reg)>>,
    ) -> Result<Block> {
        let mut m_bb = Block::new(Self::label_name_from(&bb.name).with_context(|| context!())?);
        for inst in &bb.instrs {
            let gen_insts = Self::build_instruction(
                inst,
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
        let gen_insts =
            Self::build_term_inst(&bb.term, reg_gener, regs, fmms).with_context(|| context!())?;
        m_bb.extend_insts(gen_insts);
        Ok(m_bb)
    }
}
