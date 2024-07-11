use super::super::prog::Program;
use super::*;

use anyhow::{Context, Result};

use llvm_ir::{Constant, Name};
use std::collections::HashMap;
use var::Var;

pub struct IRBuilder;

impl IRBuilder {
    #[cfg(feature = "clang_enabled")]
    #[allow(unused)]
    pub fn gen_from_clang(program: &clang_frontend::Program) -> Result<Program> {
        let llvm_module = &program.llvm;
        // dbg!(&llvm.types);
        let global_vars = Self::build_global_var(&llvm_module.global_vars)?;
        // dbg!(&global_vars);
        let funcs = Self::build_funcs(&llvm_module.functions)?;

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

    pub fn build_funcs(llvm_funcs: &[llvm_ir::Function]) -> Result<Vec<Func>> {
        let mut funcs = Vec::new();
        for f in llvm_funcs {
            // dbg!(&f);
            let args: Vec<String> = f.parameters.iter().map(|p| p.name.to_string()).collect();
            let mut reg_gener = RegGenerator::new();
            let mut regs: HashMap<Name, Reg> = HashMap::new();
            let mut stack_allocator = StackAllocator::new();
            let mut stack_slots: HashMap<Name, StackSlot> = HashMap::new();
            let entry = Self::build_entry(
                f,
                &mut stack_allocator,
                &mut stack_slots,
                &mut reg_gener,
                &mut regs,
            )?;
            let mut m_f = Func::new(f.name.to_string(), args, entry);

            for bb in Self::build_other_bbs(
                f,
                &mut stack_allocator,
                &mut stack_slots,
                &mut reg_gener,
                &mut regs,
            )? {
                m_f.push_bb(bb);
            }

            *m_f.stack_allocator_mut() = Some(stack_allocator);
            funcs.push(m_f);
        }
        Ok(funcs)
    }

    fn build_entry(
        f: &llvm_ir::Function,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Name, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Block> {
        let bb = f
            .basic_blocks
            .first()
            .ok_or(anyhow!("no basic block"))
            .with_context(|| context!())?;
        let mut insts = Vec::new();
        let mut extern_arg_start = 0;
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
                let ld_inst = LdInst::new(v_reg, extern_arg_start.into(), REG_S0);
                insts.push(ld_inst.into());
                extern_arg_start += 8;
            }
        }

        for inst in &bb.instrs {
            let gen_insts =
                Self::build_instruction(inst, stack_allocator, stack_slots, reg_gener, regs)
                    .with_context(|| context!())?;
            insts.extend(gen_insts);
        }

        insts.extend(Self::build_term_inst(&bb.term, regs)?);

        let mut entry = Block::new("entry".to_string());
        entry.extend_insts(insts);
        Ok(entry)
    }
    fn build_other_bbs(
        f: &llvm_ir::Function,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Name, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Vec<Block>> {
        let mut ret: Vec<Block> = Vec::new();
        for bb in &f.basic_blocks[1..] {
            let m_bb = Self::build_bb(bb, stack_allocator, stack_slots, reg_gener, regs)?;
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
    ) -> Result<Block> {
        let mut m_bb = Block::new(
            bb.name
                .to_string()
                .strip_prefix('%')
                .unwrap_or(&bb.name.to_string())
                .to_string(),
        );
        for inst in &bb.instrs {
            let gen_insts =
                Self::build_instruction(inst, stack_allocator, stack_slots, reg_gener, regs)
                    .with_context(|| context!())?;
            m_bb.extend_insts(gen_insts);
        }
        let gen_insts = Self::build_term_inst(&bb.term, regs).with_context(|| context!())?;
        m_bb.extend_insts(gen_insts);
        Ok(m_bb)
    }
}
