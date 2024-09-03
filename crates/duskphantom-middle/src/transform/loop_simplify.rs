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

use anyhow::{Ok, Result};

use crate::{
    analysis::loop_tools::{LoopForest, LoopPtr},
    ir::{
        instruction::{downcast_mut, misc_inst::Phi, InstType},
        Instruction, Operand,
    },
    transform::loop_optimization::loop_forest_post_order,
    IRBuilder,
};

type IRBuilderWraper = Pin<Box<IRBuilder>>;

pub struct LoopSimplifier<'a> {
    ir_builder: &'a mut IRBuilderWraper,
}

impl<'a> LoopSimplifier<'a> {
    pub fn new(ir_builder: &'a mut IRBuilderWraper) -> LoopSimplifier {
        Self { ir_builder }
    }

    pub fn run(&mut self, loop_forest: &mut LoopForest) -> Result<()> {
        loop_forest_post_order(loop_forest, |x| self.simplify_one_loop(x))
    }

    fn simplify_one_loop(&mut self, lo: LoopPtr) -> Result<()> {
        if lo.pre_header.is_none() {
            self.insert_preheader(lo)?;
        }

        self.insert_unique_backedge_block(lo)?;
        Ok(())
    }

    fn insert_unique_backedge_block(&mut self, mut lo: LoopPtr) -> Result<()> {
        let head = lo.head;
        let backedge_blocks_index = head
            .get_pred_bb()
            .iter()
            .enumerate()
            .filter_map(|(index, &bb)| {
                if bb != lo.pre_header.unwrap() {
                    Some(index)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if backedge_blocks_index.len() == 1 {
            return Ok(());
        }

        let mut unique_backedge_block = self
            .ir_builder
            .new_basicblock("uni_backedge_".to_owned() + &lo.head.name);
        let mut tail = self.ir_builder.get_br(None);
        unique_backedge_block.push_back(tail);

        let mut inst = head.get_first_inst();
        while let InstType::Phi = inst.get_type() {
            let phi = downcast_mut::<Phi>(inst.as_mut());

            let incoming_values = backedge_blocks_index
                .iter()
                .map(|index| phi.get_incoming_values()[*index].clone())
                .collect::<Vec<_>>();

            let new_phi = self
                .ir_builder
                .get_phi(phi.get_value_type(), incoming_values);

            tail.insert_before(new_phi);

            for (i, index) in backedge_blocks_index.iter().enumerate() {
                phi.get_incoming_values_mut().remove(index - i);
                unsafe { phi.get_manager_mut().remove_operand(index - i) };
            }
            phi.add_incoming_value(Operand::Instruction(new_phi), unique_backedge_block);

            if let Some(next) = inst.get_next() {
                inst = next;
            }
        }

        backedge_blocks_index
            .into_iter()
            .map(|index| head.get_pred_bb()[index])
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|mut bb| {
                bb.replace_succ_bb_only(head, unique_backedge_block);
            }); //

        unique_backedge_block.set_true_bb(head);
        lo.blocks.insert(unique_backedge_block);

        Ok(())
    }

    fn insert_preheader(&mut self, mut lo: LoopPtr) -> Result<()> {
        let header = lo.head;

        // 获得不在循环中的bb和对应的index
        let out_bb = header
            .get_pred_bb()
            .iter()
            .enumerate()
            .filter_map(|(index, bb)| {
                if !lo.is_in_loop(bb) {
                    Some((index, *bb))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if out_bb.len() == 1 && out_bb[0].1.get_succ_bb().len() == 1 {
            lo.pre_header = Some(out_bb[0].1);
            return Ok(());
        }

        let mut preheader = self
            .ir_builder
            .new_basicblock("preheader".to_string() + &header.name);
        let out_bb_index = out_bb
            .into_iter()
            .map(|(index, mut out_bb)| {
                out_bb.replace_succ_bb_only(header, preheader);
                index
            })
            .collect::<Vec<_>>();

        preheader.set_true_bb(header);

        let mut pre_header_jump = self.ir_builder.get_br(None);
        preheader.push_back(pre_header_jump);
        // 构建对应的phi结点
        for mut phi in header.iter() {
            if InstType::Phi != phi.get_type() {
                break;
            }

            let phi = downcast_mut::<Phi>(phi.as_mut());
            let incoming_values = out_bb_index
                .iter()
                .map(|&index| phi.get_incoming_values()[index].clone())
                .collect::<Vec<_>>();

            out_bb_index.iter().enumerate().for_each(|(i, index)| {
                phi.get_incoming_values_mut().remove(index - i);
                unsafe { phi.get_manager_mut().remove_operand(index - i) };
            });

            let new_phi = self
                .ir_builder
                .get_phi(phi.get_value_type(), incoming_values);
            pre_header_jump.insert_before(new_phi);

            phi.add_incoming_value(Operand::Instruction(new_phi), preheader);
        }

        // 如果是子循环，则preheader会存在上层循环中
        if let Some(mut plo) = lo.parent_loop {
            plo.blocks.insert(preheader);
        }

        lo.pre_header = Some(preheader);

        Ok(())
    }
}
