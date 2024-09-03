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

use instruction::{downcast_mut, misc_inst::Phi, InstType};

use super::*;

pub type BBPtr = ObjPtr<BasicBlock>;

/// The organization structure of the instructions inside the basic block is a double linked list.
/// the last instruction must be br or ret.
pub struct BasicBlock {
    pub ir_builder: ObjPtr<IRBuilder>,

    pub name: String,

    pub id: usize,

    pub depth: usize,

    /// The head instruction of the `BasicBlock`,
    /// which is used to unify the insertion operation and has no actual meaning.
    /// Logical structure of the `BasicBlock` is a two-way non-circular linked list,
    /// but in actual implementation, it is a two-way circular linked list.
    head_inst: InstPtr,

    /// The predecessor `BasicBlock` of the `BasicBlock`.
    /// The number of predecessor `BasicBlocks` can theoretically be 0 to infinity.
    /// When the number of predecessor `BasicBlocks` is 0,
    /// the `BasicBlock` is the function entry `BasicBlock` or an unreachable `BasicBlock`.
    pred_bbs: Vec<BBPtr>,

    /// The successor `BasicBlock` of the `BasicBlock`.
    /// The number of successor `BasicBlocks` can theoretically be 0, 1, and 2:
    /// 1. When the number of successor `BasicBlocks` is 0, the `BasicBlock` is the function exit `BasicBlock`.
    /// 2. When the number of successor `BasicBlocks` is 1, the last instruction of the `BasicBlock` is an unconditional jump instruction.
    /// 3. When the number of successor `BasicBlocks` is 2, the last instruction of the `BasicBlock` is a conditional jump instruction.
    ///    + The `BasicBlock` with index 0 is the `BasicBlock` to jump to when the condition is true.
    ///    + The `BasicBlock` with index 1 is the `BasicBlock` to jump to when the condition is false.
    succ_bbs: Vec<BBPtr>,
}

impl BasicBlock {
    pub fn new(name: String, mut ir_builder: ObjPtr<IRBuilder>) -> Self {
        let head_inst = ir_builder.new_head();
        Self {
            ir_builder,
            name,
            id: 0,
            depth: 0,
            head_inst,
            pred_bbs: Vec::new(),
            succ_bbs: Vec::new(),
        }
    }

    /// # Safety
    ///
    /// Inits `BasicBlock` after memory allocation.
    ///
    /// This is an ugly code that is a compromise in design. You should not call this function.
    pub unsafe fn init_bb(mut bb: BBPtr, id: usize) {
        let mut head = bb.head_inst;
        bb.head_inst.set_prev(head);
        bb.head_inst.set_next(head);
        head.set_parent_bb(bb);
        bb.id = id;
    }

    /// Returns `True` if the `BasicBlock` is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.head_inst.is_last()
    }

    /// Gets first instruction in the `BasicBlock`.
    ///
    /// # Panics
    /// Please make sure the current basic block is not empty, otherwise it will panic.
    #[inline]
    pub fn get_first_inst(&self) -> InstPtr {
        self.head_inst.get_next().unwrap()
    }

    /// Gets the last instruction in the `BasicBlock`.
    ///
    /// # Panics
    /// Please make sure the current basic block is not empty, otherwise it will panic.
    #[inline]
    pub fn get_last_inst(&self) -> InstPtr {
        self.head_inst.get_prev().unwrap()
    }

    /// Appends a new instruction to the end of the `BasicBlock`.
    #[inline]
    pub fn push_back(&mut self, inst: InstPtr) {
        self.head_inst.insert_before(inst);
    }

    /// Appends a new instruction to the beginning of the `BasicBlock`.
    #[inline]
    pub fn push_front(&mut self, inst: InstPtr) {
        self.head_inst.insert_after(inst);
    }

    /// Returns `True` if the `BasicBlock` is the function entry `BasicBlock`.
    #[inline]
    pub fn is_entry(&self) -> bool {
        self.pred_bbs.is_empty()
    }

    /// Returns `True` if the `BasicBlock` is the function exit `BasicBlock`.
    #[inline]
    pub fn is_exit(&self) -> bool {
        self.succ_bbs.is_empty()
    }

    /// Gets the predecessor `BasicBlock` of the `BasicBlock`.
    #[inline]
    pub fn get_pred_bb(&self) -> &Vec<BBPtr> {
        &self.pred_bbs
    }

    /// Gets the successor `BasicBlock` of the `BasicBlock`.
    #[inline]
    pub fn get_succ_bb(&self) -> &Vec<BBPtr> {
        &self.succ_bbs
    }

    /// Remove current block.
    /// This removes self from successor's predecessor list and phi operand.
    ///
    /// # Panics
    /// Please make sure this block is unreachable!
    pub fn remove_self(&mut self) {
        for succ in self.succ_bbs.iter() {
            succ.clone().remove_pred_bb(ObjPtr::new(self));
        }
    }

    /// Replace `preds => [ self -> ... ] => succs` with `preds => [ entry -> ... ] => succs`. This:
    /// - add preds to entry's predecessor list
    /// - remove preds from self's predecessor list
    /// - replaces self to entry in predecessor's successor list
    /// - replaces self to entry in function entry
    ///
    /// # Safety
    /// Entry should be different from self, and contain no "phi" instructions.
    pub fn replace_entry(&mut self, mut entry: BBPtr, mut func: FunPtr) {
        for pred in self.pred_bbs.iter_mut() {
            entry.pred_bbs.push(*pred);
            for pred_succ in pred.succ_bbs.iter_mut() {
                if pred_succ.id == self.id {
                    *pred_succ = entry;
                }
            }
        }
        self.pred_bbs.clear();
        if func.entry.is_some() && func.entry.unwrap().id == self.id {
            func.entry = Some(entry);
        }
    }

    /// Replace `preds => [ ... -> self ] => succs` with `preds => [ ... -> exit ] => succs`. This:
    /// - add succs to exit's successor list
    /// - remove succs from self's successor list
    /// - replaces self -> exit in successor's predecessor list
    /// - replaces self -> exit in successor's phi operand
    ///
    /// # Safety
    /// Exit should be different from self.
    pub fn replace_exit(&mut self, mut exit: BBPtr) {
        for succ in self.succ_bbs.iter() {
            exit.succ_bbs.push(*succ);
            succ.clone().replace_pred_bb(ObjPtr::new(self), exit);
        }
        self.succ_bbs.clear();
    }

    /// Remove a predecessor of this block.
    pub fn remove_pred_bb(&mut self, pred: BBPtr) {
        // Remove pred bb
        self.pred_bbs.retain(|x| x.id != pred.id);

        // Remove phi operand
        for mut inst in self.iter() {
            if inst.get_type() == InstType::Phi {
                let inst = downcast_mut::<Phi>(inst.as_mut());
                inst.remove_incoming_value(pred.id);

                // If phi has only one operand, replace with the operand
                if inst.get_incoming_values().len() == 1 {
                    let only_op = inst.get_incoming_values()[0].0.clone();
                    inst.replace_self(&only_op);
                }
            }
        }
    }

    /// Replace a predecessor of this block.
    pub fn replace_pred_bb(&mut self, from: BBPtr, to: BBPtr) {
        // Replace pred bb
        for pred in self.pred_bbs.iter_mut() {
            if pred.id == from.id {
                *pred = to;
            }
        }

        // Replace phi operand
        for mut inst in self.iter() {
            if inst.get_type() == InstType::Phi {
                let inst = downcast_mut::<Phi>(inst.as_mut());
                inst.replace_incoming_value(from, to);
            }
        }
    }

    /// Replace successor with given mapping.
    /// TODO: rename me
    pub fn replace_succ_bb_only(&mut self, mut from: BBPtr, mut to: BBPtr) {
        if let Some(index) = self.succ_bbs.iter().enumerate().find_map(|(index, bb)| {
            if bb.id == from.id {
                Some(index)
            } else {
                None
            }
        }) {
            from.pred_bbs.retain(|x| x.id != self.id);
            self.succ_bbs[index] = to;
            to.pred_bbs.push(ObjPtr::new(self))
        }
    }

    /// Sets which `BasicBlock` to jump to when the condition is true.
    ///
    /// # Panics
    /// Don't replace existing true `BasicBlock` with this method.
    pub fn set_true_bb(&mut self, mut bb: BBPtr) {
        let self_ptr = ObjPtr::new(self);
        if self.succ_bbs.is_empty() {
            self.succ_bbs.push(bb);
        } else {
            panic!("The true bb already exists");
        }
        bb.pred_bbs.push(self_ptr);
    }

    /// Sets which `BasicBlock` to jump to when the condition is false.
    ///
    /// # Panics
    /// You should set the true `BasicBlock` before use this method.
    /// Don't replace existing false `BasicBlock` with this method.
    pub fn set_false_bb(&mut self, mut bb: BBPtr) {
        let self_ptr = ObjPtr::new(self);
        if self.succ_bbs.len() == 1 {
            self.succ_bbs.push(bb);
        } else {
            panic!("The false bb already exists");
        }
        bb.pred_bbs.push(self_ptr);
    }

    /// Remove basic block to jump to when the condition is false.
    /// This will only execute when false bb exists.
    pub fn remove_false_bb(&mut self) {
        let self_ptr = ObjPtr::new(self);
        if self.succ_bbs.len() == 2 {
            let mut next = self.succ_bbs[1];
            next.remove_pred_bb(self_ptr);
            self.succ_bbs.pop();
        }
    }

    /// Remove basic block to jump to when the condition is true.
    /// This will only execute when false bb exists.
    pub fn remove_true_bb(&mut self) {
        let self_ptr = ObjPtr::new(self);
        if self.succ_bbs.len() == 2 {
            let mut next = self.succ_bbs[0];
            next.remove_pred_bb(self_ptr);
            self.succ_bbs.remove(0);
        }
    }

    /// Returns a iterator of the `BasicBlock`.
    /// The iterator yields the `InstPtr` of the `BasicBlock` except the head instruction.
    pub fn iter(&self) -> BasicBlockIterator {
        BasicBlockIterator {
            cur: self.head_inst,
            next: self.head_inst.get_next(),
        }
    }

    /// Returns a reverse iterator of the `BasicBlock`.
    /// The iterator yields the `InstPtr` of the `BasicBlock` except the head instruction.
    pub fn iter_rev(&self) -> BasicBlockIteratorRev {
        BasicBlockIteratorRev {
            cur: self.head_inst,
            prev: self.head_inst.get_prev(),
        }
    }

    pub fn gen_llvm_ir(&self) -> String {
        let mut ir = String::new();
        ir += &format!("{}:\n", self.name);
        for inst in self.iter() {
            ir += &inst.gen_llvm_ir();
            ir += "\n";
        }
        ir + "\n"
    }
}

impl Display for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{}", self.name)
    }
}

impl Extend<InstPtr> for BasicBlock {
    fn extend<T: IntoIterator<Item = InstPtr>>(&mut self, iter: T) {
        iter.into_iter().for_each(|inst| {
            self.push_back(inst);
        });
    }
}

impl PartialEq for BasicBlock {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialOrd for BasicBlock {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for BasicBlock {}

impl Ord for BasicBlock {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

pub struct BasicBlockIterator {
    cur: InstPtr,
    next: Option<InstPtr>,
}

impl Iterator for BasicBlockIterator {
    type Item = InstPtr;
    fn next(&mut self) -> Option<Self::Item> {
        self.cur = self.next?;
        self.next = self.cur.get_next();
        Some(self.cur)
    }
}

pub struct BasicBlockIteratorRev {
    cur: InstPtr,
    prev: Option<InstPtr>,
}

impl Iterator for BasicBlockIteratorRev {
    type Item = InstPtr;
    fn next(&mut self) -> Option<Self::Item> {
        self.cur = self.prev?;
        self.prev = self.cur.get_prev();
        Some(self.cur)
    }
}
