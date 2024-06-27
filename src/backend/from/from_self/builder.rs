use std::collections::HashMap;

use crate::backend::*;

use crate::middle;

use crate::utils::mem::ObjPtr;

use anyhow::Result;
use llvm_ir::Name;
use var::{ArrVar, FloatVar, IntVar, PrimVar, Var};

pub struct IRBuilder;

impl IRBuilder {
    pub fn gen_from_self(program: &middle::Program) -> Result<Program> {
        let llvm_module = &program.module;
        // dbg!(&llvm.types);
        let global_vars = Self::build_global_var(&llvm_module.global_variables)?;
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

    pub fn build_global_var(llvm_global_vars: &Vec<middle::ir::GlobalPtr>) -> Result<Vec<Var>> {
        let mut global_vars = Vec::new();

        for global_var in llvm_global_vars {
            // dbg!(&global_var);
            let name = &global_var.name.to_string(); // 这里的 name 是不带 @ 的
            dbg!(&name);
            match &global_var.initializer {
                middle::ir::Constant::Int(value) => {
                    let var = Var::Prim(PrimVar::IntVar(IntVar {
                        name: name.to_string(),
                        init: Some(*value as i32),
                        is_const: false, // TODO
                    }));
                    global_vars.push(var);
                }
                middle::ir::Constant::Float(value) => {
                    let var = Var::Prim(PrimVar::FloatVar(FloatVar {
                        name: name.to_string(),
                        init: Some(*value as f32),
                        is_const: false,
                    }));
                    global_vars.push(var);
                }
                middle::ir::Constant::Bool(value) => {
                    let var = Var::Prim(PrimVar::IntVar(IntVar {
                        name: name.to_string(),
                        init: Some(*value as i32),
                        is_const: false,
                    }));
                    global_vars.push(var);
                }
                middle::ir::Constant::Array(arr) => {
                    match arr.first().with_context(|| context!())? {
                        // 不可能出现: arr 是混合的
                        middle::ir::Constant::Int(_) => {
                            let mut init = Vec::new();
                            for (index, con) in arr.iter().enumerate() {
                                if let middle::ir::Constant::Int(value) = con {
                                    init.push((index, *value as i32 as u32)); // FIXME 这里 i32 和 u32 注意
                                } else {
                                    unreachable!();
                                }
                            }
                            let arr_var = Var::IntArr(ArrVar::<u32> {
                                name: name.to_string(),
                                capacity: arr.len(),
                                init,
                                is_const: false,
                            });
                            global_vars.push(arr_var);
                        }
                        middle::ir::Constant::Float(_) => {
                            let mut init = Vec::new();
                            for (index, con) in arr.iter().enumerate() {
                                if let middle::ir::Constant::Float(value) = con {
                                    init.push((index, *value as f32));
                                } else {
                                    unreachable!();
                                }
                            }
                            let arr_var = Var::FloatArr(ArrVar::<f32> {
                                name: name.to_string(),
                                capacity: arr.len(),
                                init,
                                is_const: false, /* TODO */
                            });
                            global_vars.push(arr_var);
                        }
                        middle::ir::Constant::Bool(_) => {
                            let mut init = Vec::new();
                            for (index, con) in arr.iter().enumerate() {
                                if let middle::ir::Constant::Bool(value) = con {
                                    init.push((index, *value as i32 as u32)); // FIXME 这里注意一下
                                } else {
                                    unreachable!();
                                }
                            }
                            let arr_var = Var::IntArr(ArrVar::<u32> {
                                name: name.to_string(),
                                capacity: arr.len(),
                                init,
                                is_const: false,
                            });
                            global_vars.push(arr_var);
                        }
                        _ => {
                            unreachable!();
                        }
                    }
                }
            }
        }
        Ok(global_vars)
    }

    pub fn build_funcs(llvm_funcs: &Vec<middle::ir::FunPtr>) -> Result<Vec<Func>> {
        let mut funcs = Vec::new();
        for f in llvm_funcs {
            // dbg!(&f);
            let args: Vec<String> = f.params.iter().map(|p| p.name.to_string()).collect();
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

    fn build_other_bbs(
        f: &ObjPtr<middle::ir::Function>,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Name, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Vec<Block>> {
        let mut ret: Vec<Block> = Vec::new();
        for ptr_bb in f.dfs_iter() {
            let m_bb = Self::build_bb(&ptr_bb, stack_allocator, stack_slots, reg_gener, regs)?;
            ret.push(m_bb);
        }
        Ok(ret)
    }

    fn build_bb(
        bb: &ObjPtr<middle::ir::BasicBlock>,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Name, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Block> {
        let mut m_bb = Block::new(bb.name.clone());
        for inst in bb.iter() {
            let gen_insts =
                Self::build_instruction(&inst, stack_allocator, stack_slots, reg_gener, regs)
                    .with_context(|| context!())?;
            m_bb.extend_insts(gen_insts);
        }
        // FIXME
        let gen_insts =
            Self::build_term_inst(&bb.get_last_inst(), regs).with_context(|| context!())?;
        m_bb.extend_insts(gen_insts);
        Ok(m_bb)
    }

    fn build_entry(
        f: &ObjPtr<middle::ir::Function>,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Name, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Name, Reg>,
    ) -> Result<Block> {
        // let bb = f.basic_blocks.first().expect("func must have entry");
        let bb = f.entry.with_context(|| context!())?;
        let mut insts: Vec<Inst> = Vec::new();
        // let mut insts = Vec::new();
        for (i, param) in f.params.iter().enumerate() {
            if i <= 7 {
                let reg = if IRBuilder::is_ty_int(&param.value_type) {
                    Reg::new(REG_A0.id() + i as u32, true)
                } else if IRBuilder::is_ty_float(&param.value_type) {
                    Reg::new(REG_FA0.id() + i as u32, true)
                } else {
                    // TODO 还有 Array 之类的
                    unimplemented!();
                };
                regs.insert(param.name.clone().into(), reg);
            } else {
                unimplemented!();
            }
        }

        for inst in bb.iter() {
            let gen_insts =
                Self::build_instruction(&inst, stack_allocator, stack_slots, reg_gener, regs)
                    .with_context(|| context!())?;
            insts.extend(gen_insts);
        }
        insts.extend(Self::build_term_inst(&bb.get_last_inst(), regs)?);
        let mut entry = Block::new("entry".to_string());
        entry.extend_insts(insts);
        Ok(entry)
    }
}
