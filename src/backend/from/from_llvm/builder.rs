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
            // count stack size,
            // let stack_size = stack_allocator.allocated();
            // // align to 16
            // let stack_size = if stack_size % 16 == 0 {
            //     stack_size
            // } else {
            //     stack_size - stack_size % 16 + 16
            // };
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
        let bb = f.basic_blocks.first().expect("func must have entry");
        let mut insts = Vec::new();
        for (i, param) in f.parameters.iter().enumerate() {
            if i <= 7 {
                let reg = if IRBuilder::is_ty_int(&param.ty) {
                    Reg::new(REG_A0.id() + i as u32, true)
                } else if IRBuilder::is_ty_float(&param.ty) {
                    Reg::new(REG_FA0.id() + i as u32, true)
                } else {
                    unimplemented!();
                };
                regs.insert(param.name.clone(), reg);
            } else {
                unimplemented!();
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
