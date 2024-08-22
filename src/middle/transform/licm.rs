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

use std::pin::Pin;

use crate::middle::{
    analysis::{
        loop_tools::{LoopForest, LoopPtr},
        memory_ssa::MemorySSA,
    },
    ir::{instruction::InstType, BBPtr, IRBuilder, InstPtr, Operand},
};
use anyhow::{Ok, Result};

use super::loop_optimization::loop_forest_post_order;

type IRBuilderWraper = Pin<Box<IRBuilder>>;

pub struct LICM<'a, 'b> {
    _ir_builder: &'a mut IRBuilderWraper,
    memory_ssa: &'a mut MemorySSA<'b>,
}

impl<'a, 'b> LICM<'a, 'b> {
    pub fn new(
        _ir_builder: &'a mut IRBuilderWraper,
        memory_ssa: &'a mut MemorySSA<'b>,
    ) -> LICM<'a, 'b> {
        Self {
            _ir_builder,
            memory_ssa,
        }
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
            self.licm_inst_trace(lo, inst, preheader)?
        }
        Ok(())
    }

    fn licm_inst_trace(&mut self, lo: LoopPtr, mut inst: InstPtr, preheader: BBPtr) -> Result<()> {
        if
        // 防止递归之后得到循环外的inst
        lo.is_in_loop(&inst.get_parent_bb().unwrap())
        // 以下指令暂不考虑
        && !matches!(
            inst.get_type(),
            InstType::Br
            | InstType::Alloca
            | InstType::Store
            | InstType::Call
            | InstType::Ret
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
        // 对于 Load 指令，所使用的 MemoryDef 必须在循环外
        && self.memory_ssa.get_inst_node(inst).map_or(true, |node| {
            !lo.is_in_loop(&self.memory_ssa.get_node_block(node.get_use_node()).unwrap())
        }) {
            unsafe {
                inst.move_self();
            }
            // 无需移动 MemorySSA 节点，反正 Load 不会被别的用
            preheader.get_last_inst().insert_before(inst);

            inst.get_user()
                .iter()
                .try_for_each(|&user| self.licm_inst_trace(lo, user, preheader))?
        }
        Ok(())
    }
}
