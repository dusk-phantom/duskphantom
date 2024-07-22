use anyhow::Result;
use std::collections::HashMap;
use var::{ArrVar, FloatVar, IntVar, PrimVar, Var};

use super::Address;
use crate::backend::*;
use crate::middle;
use crate::utils::mem::ObjPtr;

pub struct IRBuilder;

impl IRBuilder {
    pub fn new_var(ty: &middle::ir::ValueType, reg_gener: &mut RegGenerator) -> Result<Reg> {
        let dst_reg = match ty {
            middle::ir::ValueType::Int
            | middle::ir::ValueType::Bool
            | middle::ir::ValueType::Pointer(_) => reg_gener.gen_virtual_usual_reg(),
            middle::ir::ValueType::Float => reg_gener.gen_virtual_float_reg(),
            _ => return Err(anyhow!("phi can't be void/array")).with_context(|| context!()),
        };
        Ok(dst_reg)
    }
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
                middle::ir::Constant::SignedChar(value) => {
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

    #[allow(unused)]
    pub fn build_funcs(
        self_funcs: &Vec<middle::ir::FunPtr>,
        fmms: &mut HashMap<Fmm, FloatVar>,
    ) -> Result<Vec<Func>> {
        let _ = self_funcs;
        todo!()
    }

    pub fn build_func(
        self_func: &middle::ir::FunPtr,
        fmms: &mut HashMap<Fmm, FloatVar>,
    ) -> Result<Func> {
        let args: Vec<_> = self_func
            .params
            .iter()
            .map(|p| p.name.to_string())
            .collect();

        let mut insert_back_for_remove_phi = HashMap::new();
        let mut reg_gener = RegGenerator::new();
        let mut regs: HashMap<Address, Reg> = HashMap::new();
        let mut stack_allocator = StackAllocator::new();
        let mut stack_slots: HashMap<Address, StackSlot> = HashMap::new();

        // entry
        let (entry, caller_reg_stack) = Self::build_entry(
            self_func,
            &mut stack_allocator,
            &mut stack_slots,
            &mut reg_gener,
            &mut regs,
            fmms,
            &mut insert_back_for_remove_phi,
        )?;

        let mut m_f = Func::new(self_func.name.to_string(), args, entry);

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

        *m_f.caller_regs_stack_mut() = Some(caller_reg_stack.try_into()?);
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

        // insert back to bbs to process phi
        let mut bbs_mut = m_f
            .iter_bbs_mut()
            .map(|bb| (bb.label().to_string(), bb))
            .collect::<HashMap<String, &mut Block>>();
        for (bb_name, insert_back) in insert_back_for_remove_phi {
            let bb = bbs_mut
                .get_mut(&bb_name)
                .ok_or(anyhow!("").context(context!()))?;
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

    /// caller_regs_stack 是在 build 单个 func 的时候确定的
    /// 这里一定要放在 build_funcs 之后, 因为这个时候，所有的函数的 caller_regs_stack 才会被计算好
    #[allow(unused)]
    fn prepare_max_callee_regs_stack(funcs: &mut Vec<Func>) {
        let name_func: HashMap<String, u32> = funcs
            .iter()
            .map(|f| (f.name().to_string(), f.caller_regs_stack()))
            .collect();

        for f in funcs {
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
    }

    fn build_other_bbs(
        func: &ObjPtr<middle::ir::Function>,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Address, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
        insert_back_for_remove_phi: &mut HashMap<String, Vec<(middle::ir::Operand, Reg)>>,
    ) -> Result<Vec<Block>> {
        // let mut blocks: Vec<Block> = Vec::new();
        // for ptr_bb in f.dfs_iter() {
        //     let m_bb = Self::build_bb(&ptr_bb, stack_allocator, stack_slots, reg_gener, regs)?;
        //     blocks.push(m_bb);
        // }
        // Ok(blocks)
        func.dfs_iter()
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
        let label = format!(".LBB{}", bb.as_ref() as *const _ as Address);
        let mut m_bb = Block::new(label);
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
        // let gen_insts = Self::build_term_inst(&bb.get_last_inst(), regs, reg_gener, fmms)
        //     .with_context(|| context!())?;
        // m_bb.extend_insts(gen_insts);
        Ok(m_bb)
    }

    fn build_entry(
        func: &ObjPtr<middle::ir::Function>,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Address, StackSlot>,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
        insert_back_for_remove_phi: &mut HashMap<String, Vec<(middle::ir::Operand, Reg)>>,
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
                | middle::ir::ValueType::SignedChar
                | middle::ir::ValueType::Bool
                | middle::ir::ValueType::Int => true,
            };

            let v_reg = reg_gener.gen_virtual_reg(is_usual);
            regs.insert(param.as_ref() as *const _ as Address, v_reg); // 参数绑定寄存器

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
        // // 上面 bb.iter 会包含这个 term_inst
        // insts.extend(Self::build_term_inst(
        //     &bb.get_last_inst(),
        //     regs,
        //     reg_gener,
        //     fmms,
        // )?);

        let label = format!(".LBB{}", bb.as_ref() as *const _ as Address);
        let mut entry = Block::new(label);
        entry.extend_insts(insts);
        let caller_regs_stack = usize::try_from(caller_regs_stack)?;
        Ok((entry, caller_regs_stack))
    }
}
