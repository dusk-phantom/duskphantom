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

use super::*;
/// 处理调用者保存和被调用者保存
pub fn handle_caller_callee(func: &mut Func) -> Result<()> {
    handle_caller_save(func)?;
    handle_callee_save(func)?;
    Ok(())
}
fn take_ssa(f: &mut Func) -> Result<StackAllocator> {
    f.stack_allocator_mut()
        .take()
        .ok_or(anyhow!("stack allocator is none"))
}

#[allow(unused)]
fn handle_caller_save(func: &mut Func) -> Result<()> {
    let mut ssa = take_ssa(func)?;
    let mut available_ss = Vec::new();

    let reg_lives = Func::reg_lives(func)?;
    for bb in func.iter_bbs_mut() {
        let mut new_insts_rev: Vec<Inst> = vec![];
        let mut alive_regs = reg_lives.live_outs(bb).clone();
        alive_regs.retain(|r| r.is_caller_save());
        for inst in bb.insts_mut().iter_mut().rev() {
            if let Inst::Call(call) = inst {
                let mut to_save = alive_regs.clone().into_iter().collect::<Vec<_>>();
                to_save.retain(|r| !call.defs().contains(&r));
                for i in (available_ss.len()..to_save.len()) {
                    available_ss.push(ssa.alloc(8));
                }
                for (r, ss) in to_save.iter().zip(available_ss.iter()) {
                    new_insts_rev.push(LoadInst::new(*r, *ss).with_8byte().into());
                }
                new_insts_rev.push(inst.clone());
                for (r, ss) in to_save.iter().zip(available_ss.iter()) {
                    new_insts_rev.push(StoreInst::new(*ss, *r).with_8byte().into());
                }
            } else {
                new_insts_rev.push(inst.clone());
            }
            alive_regs.retain(|r| !inst.defs().contains(&r));
            alive_regs.extend(inst.uses().iter().cloned());
            alive_regs.retain(|r| r.is_caller_save());
            // update alive_regs
        }
        new_insts_rev.reverse();
        bb.insts_mut().clear();
        bb.insts_mut().extend(new_insts_rev);
    }

    func.stack_allocator_mut().replace(ssa);
    Ok(())
}

#[allow(unused)]
fn handle_callee_save(func: &mut Func) -> Result<()> {
    let mut ssa = take_ssa(func)?;

    let mut available_ss: Vec<StackSlot> = Vec::new();
    let mut regs = func.regs();
    regs.retain(|r| r.is_callee_save());
    (0..regs.len()).for_each(|_| available_ss.push(ssa.alloc(8)));

    let mut insert_front: Vec<Inst> = vec![];
    for (r, ss) in regs.iter().zip(available_ss.iter()) {
        insert_front.push(StoreInst::new(*ss, *r).with_8byte().into());
    }

    let mut insert_back: Vec<Inst> = vec![];
    for (r, ss) in regs.iter().zip(available_ss.iter()) {
        insert_back.push(LoadInst::new(*r, *ss).with_8byte().into());
    }

    let entry = func.entry_mut();
    entry.insts_mut().splice(0..0, insert_front.iter().cloned());

    let exits = func.exit_bbs_mut();
    for bb in exits {
        for inst in insert_back.iter() {
            bb.insert_before_term(inst.clone())?;
        }
    }

    func.stack_allocator_mut().replace(ssa);
    Ok(())
}
