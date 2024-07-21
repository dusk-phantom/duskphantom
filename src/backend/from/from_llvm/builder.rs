use super::super::prog::Program;
use super::*;

use anyhow::{Context, Result};

use llvm_ir::{Constant, Name};
use std::collections::HashMap;
use var::{FloatVar, Var};

pub struct IRBuilder;

impl IRBuilder {
    #[cfg(feature = "clang_enabled")]
    #[allow(unused)]
    pub fn gen_from_clang(program: &clang_frontend::Program) -> Result<Program> {
        use var::FloatVar;

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

    pub fn build_global_var(
        llvm_global_vars: &[llvm_ir::module::GlobalVariable],
    ) -> Result<Vec<Var>> {
        let mut global_vars = Vec::new();
        for global_var in llvm_global_vars {
            // dbg!(&global_var);
            let name = &global_var.name.to_string()[1..];
            if let Some(init) = &global_var.initializer {
                // dbg!(&init);
                let c = init.as_ref().to_owned();
                match c {
                    Constant::Int { bits: _, value } => {
                        let var = var::Var::Prim(var::PrimVar::IntVar(var::IntVar {
                            name: name.to_string(),
                            init: Some(value as i32),
                            is_const: false,
                        }));
                        global_vars.push(var);
                    }
                    Constant::Float(f) => match f {
                        llvm_ir::constant::Float::Single(f) => {
                            let var = var::Var::Prim(var::PrimVar::FloatVar(var::FloatVar {
                                name: name.to_string(),
                                init: Some(f),
                                is_const: false,
                            }));
                            global_vars.push(var);
                        }
                        llvm_ir::constant::Float::Double(_) => {
                            unimplemented!("double float");
                            // let var = var::Var::Prim(var::PrimVar::FloatVar(var::FloatVar {
                            //     name: name.to_string(),
                            //     init: Some(f),
                            //     is_const: false,
                            // }));
                            // global_vars.push(var);
                        }
                        _ => {
                            unreachable!();
                        }
                    },
                    _ => (),
                }
            }
        }
        Ok(global_vars)
    }

    /**
     * build funcs
     */
    pub fn build_funcs(
        llvm_funcs: &[llvm_ir::Function],
        fmms: &mut HashMap<Fmm, FloatVar>,
    ) -> Result<Vec<Func>> {
        let mut funcs = Vec::new();
        for llvm_func in llvm_funcs {
            let func = Self::build_func(llvm_func, fmms)?;
            funcs.push(func);
        }
        // count max_callee_regs_stack
        Self::prepare_max_callee_regs_stack(&mut funcs);

        Ok(funcs)
    }
    pub fn build_func(
        llvm_func: &llvm_ir::Function,
        fmms: &mut HashMap<Fmm, FloatVar>,
    ) -> Result<Func> {
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

        *m_f.caller_regs_stack_mut() = Some(caller_reg_stack.try_into()?);
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
            let bb = bbs_mut.get_mut(&bb_name).unwrap();
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
        Ok(m_f)
    }

    pub fn prepare_max_callee_regs_stack(funcs: &mut Vec<Func>) {
        let name_func: HashMap<String, u32> = funcs
            .iter()
            .map(|f| (f.name().to_string(), f.caller_regs_stack()))
            .collect();
        for f in funcs {
            let mut max_callee_regs_stack = 0;
            for bb in f.iter_bbs() {
                for inst in bb.insts() {
                    if let Inst::Call(c) = inst {
                        let callee_regs_stack =
                            *name_func.get(c.func_name().as_str()).unwrap_or(&0);
                        max_callee_regs_stack =
                            std::cmp::max(max_callee_regs_stack, callee_regs_stack);
                    }
                }
            }
            *f.max_callee_regs_stack_mut() = Some(max_callee_regs_stack);
        }
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
                insert_back_for_remove_phi,
            )
            .with_context(|| context!())?;
            insts.extend(gen_insts);
        }

        insts.extend(Self::build_term_inst(&bb.term, reg_gener, regs, fmms)?);

        let mut entry = Block::new("entry".to_string());
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
