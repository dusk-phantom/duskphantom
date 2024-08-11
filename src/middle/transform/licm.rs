use std::pin::Pin;

use crate::middle::{
    analysis::loop_tools::{LoopForest, LoopPtr},
    ir::{instruction::InstType, BBPtr, IRBuilder, InstPtr, Operand},
};
use anyhow::{Ok, Result};

use super::loop_optimization::loop_forest_post_order;

type IRBuilderWraper = Pin<Box<IRBuilder>>;

pub struct LICM<'a> {
    _ir_builder: &'a mut IRBuilderWraper,
}

impl<'a> LICM<'a> {
    pub fn new(_ir_builder: &'a mut IRBuilderWraper) -> LICM {
        Self { _ir_builder }
    }

    pub fn run(&mut self, forest: &mut LoopForest) -> Result<()> {
        loop_forest_post_order(forest, |x| self.licm_one_loop(x))
    }

    fn licm_one_loop(&mut self, lo: LoopPtr) -> Result<()> {
        let preheader = lo.pre_header.unwrap();
        lo.blocks
            .iter()
            .try_for_each(|&x| self.licm_one_bb(lo, x, preheader))
    }

    fn licm_one_bb(&mut self, lo: LoopPtr, bb: BBPtr, preheader: BBPtr) -> Result<()> {
        for inst in bb.iter() {
            Self::licm_inst_trace(lo, inst, preheader)?
        }
        Ok(())
    }

    fn licm_inst_trace(lo: LoopPtr, mut inst: InstPtr, preheader: BBPtr) -> Result<()> {
        if
        // 防止递归之后得到循环外的inst
        lo.is_in_loop(&inst.get_parent_bb().unwrap())
        // 以下指令暂不考虑（没有指针分析）
        && !matches!(
            inst.get_type(),
            InstType::Br
            | InstType::Alloca
            | InstType::Load
            | InstType::Store
            | InstType::Ret
            | InstType::Call
            | InstType::Phi
            )
        // 判断是否是循环不变量，即操作数是否都在循环外 
        && inst.get_operand().iter().all(|i| {
            if let Operand::Instruction(i) = i {
                !lo.is_in_loop(&i.get_parent_bb().unwrap())
            } else {
                true
            }
            })
        {
            unsafe {
                inst.move_self();
            }
            preheader.get_last_inst().insert_before(inst);

            inst.get_user()
                .iter()
                .try_for_each(|&user| Self::licm_inst_trace(lo, user, preheader))?
        }
        Ok(())
    }
}
