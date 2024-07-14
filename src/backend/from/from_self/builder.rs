use anyhow::Result;
use std::collections::HashMap;
use var::{ArrVar, FloatVar, IntVar, PrimVar, Var};

use super::Address;
use crate::backend::*;
use crate::middle;
use crate::utils::mem::ObjPtr;

pub struct IRBuilder;

impl IRBuilder {
    pub fn gen_from_self(program: &middle::Program) -> Result<Program> {
        let self_module = &program.module;
        // dbg!(&llvm.types);
        let global_vars = Self::build_global_var(&self_module.global_variables)?;
        // dbg!(&global_vars);
        let funcs = Self::build_funcs(&self_module.functions)?;

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

    pub fn build_global_var(self_global_vars: &Vec<middle::ir::GlobalPtr>) -> Result<Vec<Var>> {
        let mut global_vars = Vec::new();

        for global_var in self_global_vars {
            // dbg!(&global_var);
            let name = &global_var.name.to_string(); // 这里的 name 是不带 @ 的
            dbg!(&name);
            match &global_var.initializer {
                middle::ir::Constant::Int(value) => {
                    let var = Var::Prim(PrimVar::IntVar(IntVar {
                        name: name.to_string(),
                        init: Some(*value),
                        is_const: false, // TODO 这个可能要删掉
                    }));
                    global_vars.push(var);
                }
                middle::ir::Constant::Float(value) => {
                    let var = Var::Prim(PrimVar::FloatVar(FloatVar {
                        name: name.to_string(),
                        init: Some(*value),
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
                        middle::ir::Constant::Bool(_) | middle::ir::Constant::Int(_) => {
                            let mut init = Vec::new();
                            for (index, con) in arr.iter().enumerate() {
                                if let middle::ir::Constant::Int(value) = con {
                                    init.push((index, *value as u32)); // FIXME 这里 i32 和 u32 注意
                                } else {
                                    return Err(anyhow!("arr can't be mixed with other-type"))
                                        .with_context(|| context!());
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
                                    init.push((index, *value));
                                } else {
                                    return Err(anyhow!(
                                        "arr can't be mixed with float and others"
                                    ))
                                    .with_context(|| context!());
                                }
                            }
                            let arr_var = Var::FloatArr(ArrVar::<f32> {
                                name: name.to_string(),
                                capacity: arr.len(),
                                init,
                                is_const: false,
                            });
                            global_vars.push(arr_var);
                        }
                        // TODO 是否有全局初始化过的二维数组 ？
                        _ => {
                            unreachable!();
                        }
                    }
                }
            }
        }
        Ok(global_vars)
    }

    pub fn build_funcs(self_funcs: &Vec<middle::ir::FunPtr>) -> Result<Vec<Func>> {
        let mut funcs = Vec::new();
        for fu in self_funcs {
            // dbg!(&f);
            let args: Vec<String> = fu.params.iter().map(|p| p.name.to_string()).collect();
            let mut reg_gener = RegGenerator::new(); // 一个 func 绑定一个 reg_gener
            let mut regs: HashMap<Address, Reg> = HashMap::new();
            let mut stack_allocator = StackAllocator::new();
            let mut stack_slots: HashMap<Address, StackSlot> = HashMap::new();
            let (entry, caller_reg_stack) = Self::build_entry(
                fu,
                &mut stack_allocator,
                &mut stack_slots,
                &mut reg_gener,
                &mut regs,
            )?;
            let mut m_f = Func::new(fu.name.to_string(), args, entry);
            *m_f.caller_regs_stack_mut() = Some(caller_reg_stack.try_into()?);
            for bb in Self::build_other_bbs(
                fu,
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

        // 看一个函数，他里面的所有的调用
        let name_func: HashMap<String, u32> = funcs
            .iter()
            .map(|f| (f.name().to_string(), f.caller_regs_stack()))
            .collect();

        for f in &mut funcs {
            let mut max_callee_regs_stack = 0;
            for bb in f.iter_bbs() {
                for inst in bb.insts() {
                    if let Inst::Call(c) = inst {
                        let callee_regs_stack = *name_func.get(c.func_name().as_str()).unwrap();
                        max_callee_regs_stack =
                            std::cmp::max(max_callee_regs_stack, callee_regs_stack);
                    }
                }
            }
            *f.max_callee_regs_stack_mut() = Some(max_callee_regs_stack);
        }

        Ok(funcs)
    }

    fn build_other_bbs(
        func: &ObjPtr<middle::ir::Function>,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Address, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
    ) -> Result<Vec<Block>> {
        // let mut blocks: Vec<Block> = Vec::new();
        // for ptr_bb in f.dfs_iter() {
        //     let m_bb = Self::build_bb(&ptr_bb, stack_allocator, stack_slots, reg_gener, regs)?;
        //     blocks.push(m_bb);
        // }
        // Ok(blocks)
        func.dfs_iter()
            .map(|ptr_bb| Self::build_bb(&ptr_bb, stack_allocator, stack_slots, reg_gener, regs))
            .collect()
    }

    fn build_bb(
        bb: &ObjPtr<middle::ir::BasicBlock>,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Address, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
    ) -> Result<Block> {
        // basic 的 label 注意一下
        let label = bb.as_ref() as *const _ as Address;
        let mut m_bb = Block::new(label.to_string());
        for inst in bb.iter() {
            let gen_insts =
                Self::build_instruction(&inst, stack_allocator, stack_slots, reg_gener, regs)
                    .with_context(|| context!())?;
            m_bb.extend_insts(gen_insts);
        }
        let gen_insts = Self::build_term_inst(&bb.get_last_inst(), regs, reg_gener)
            .with_context(|| context!())?;
        m_bb.extend_insts(gen_insts);
        Ok(m_bb)
    }

    fn build_entry(
        func: &ObjPtr<middle::ir::Function>,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Address, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
    ) -> Result<(Block, usize)> {
        let mut insts: Vec<Inst> = Vec::new();

        let mut caller_regs_stack = 0;
        let mut float_idx = 0;
        let mut usual_idx = 0;

        // 处理函数的形参
        for param in func.params.iter() {
            let is_usual: bool = match &param.value_type {
                // 返回生成的 虚拟寄存器
                middle::ir::ValueType::Void => {
                    return Err(anyhow!(
                        "it is impossible to receive void-type parameter: {}",
                        param
                    ))
                }
                middle::ir::ValueType::Array(_, _) => {
                    return Err(anyhow!("array should be pointer {}", param))
                }
                middle::ir::ValueType::Float => false,
                middle::ir::ValueType::Pointer(_)
                | middle::ir::ValueType::Bool
                | middle::ir::ValueType::Int => true,
            };

            let v_reg = reg_gener.gen_virtual_reg(is_usual);
            regs.insert(param.as_ref() as *const _ as Address, v_reg); // 直接就先绑定了寄存器

            if is_usual && usual_idx <= 7 {
                let a_reg = Reg::new(REG_A0.id() + usual_idx, is_usual);
                let mv = MvInst::new(v_reg.into(), a_reg.into());
                insts.push(mv.into());
                usual_idx += 1;
            }
            if !is_usual && float_idx <= 7 {
                let a_reg = Reg::new(REG_FA0.id() + float_idx, is_usual);
                let mv = MvInst::new(v_reg.into(), a_reg.into());
                insts.push(mv.into()); // TODO 但是 mv 指令可能有点问题, mv 是伪指令, 能不能 mv float, float ?
                float_idx += 1;
            }

            //  参数多了的情况

            if (is_usual && usual_idx > 7) || (!is_usual && float_idx > 7) {
                let ld_inst = LdInst::new(v_reg, caller_regs_stack.into(), REG_S0);
                insts.push(ld_inst.into());
                caller_regs_stack += 8;
            }
        }

        let bb = func.entry.with_context(|| context!())?; // FIXME func 的其他 blocks 是不是不包含 entry ?
        for inst in bb.iter() {
            let gen_insts =
                Self::build_instruction(&inst, stack_allocator, stack_slots, reg_gener, regs)
                    .with_context(|| context!())?;
            insts.extend(gen_insts);
        }
        insts.extend(Self::build_term_inst(&bb.get_last_inst(), regs, reg_gener)?);

        let mut entry = Block::new("entry".to_string());
        entry.extend_insts(insts);
        let caller_regs_stack = usize::try_from(caller_regs_stack)?;
        Ok((entry, caller_regs_stack))
    }
}
