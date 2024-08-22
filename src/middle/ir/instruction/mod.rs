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
pub mod binary_inst;
pub mod extend_inst;
pub mod head;
pub mod memory_op_inst;
pub mod misc_inst;
pub mod terminator_inst;
pub mod unary_inst;

pub type InstPtr = ObjPtr<Box<dyn Instruction>>;

use crate::{define_inst_type_enum, gen_common_code};
use std::any::Any;

impl Display for InstPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

define_inst_type_enum!(
    // You will never get this type
    Head,
    // Binary Operations
    Add,
    FAdd,
    Sub,
    FSub,
    Mul,
    FMul,
    UDiv,
    SDiv,
    FDiv,
    URem,
    SRem,
    // FRem,
    // Bitwise Binary Operations
    Shl,
    LShr,
    AShr,
    And,
    Or,
    Xor,
    // Unary Operations
    // FNeg,
    // Terminator Instructions
    Ret,
    Br,
    // Memory Access and Addressing Operations
    Alloca,
    Load,
    Store,
    GetElementPtr,
    // Conversion Operations
    ZextTo,
    SextTo,
    ItoFp,
    FpToI,
    // Other Operations
    ICmp,
    FCmp,
    Phi,
    Call
);

pub trait Instruction: Display {
    /// # Safety
    /// Don't call this method, use downcast_ref instead.
    unsafe fn as_any(&self) -> &dyn Any;

    /// # Safety
    /// Don't call this method, use downcast_mut instead.
    unsafe fn as_any_mut(&mut self) -> &mut dyn Any;

    /// # Safety
    /// Do not call this function directly
    fn copy_self(&self) -> Box<dyn Instruction>;

    /// Returns the type of current instruction.
    fn get_type(&self) -> InstType;

    /// Returns the manager of current instruction.
    fn get_manager(&self) -> &InstManager;

    /// Returns the manager of current instruction.
    ///
    /// # Safety
    /// You should not use this function, because it may cause unknown errors.
    unsafe fn get_manager_mut(&mut self) -> &mut InstManager;

    /// Returns the instructions that use current instruction as operand.
    #[inline]
    fn get_user(&self) -> &[InstPtr] {
        &self.get_manager().user
    }

    /// Returns the instructions that use current instruction as operand.
    ///
    /// # Safety
    /// You should not use this function, because it may cause unknown errors.
    #[inline]
    unsafe fn get_user_mut(&mut self) -> &mut Vec<InstPtr> {
        &mut self.get_manager_mut().user
    }

    /// Returns the operands of current instruction.
    #[inline]
    fn get_operand(&self) -> &[Operand] {
        &self.get_manager().operand
    }

    /// Add an operand to the instruction.
    fn add_operand(&mut self, operand: Operand) {
        unsafe {
            self.get_manager_mut().add_operand(operand);
        }
    }

    /// Set the operand of cur inst by index and operand (safe and interface).
    ///
    /// # Panics
    /// It will panic with index out of range!
    #[inline]
    fn set_operand(&mut self, index: usize, operand: Operand) {
        unsafe {
            self.get_manager_mut().set_operand(index, operand);
        }
    }

    /// Replace all occurence of `from` to `to`.
    #[inline]
    fn replace_operand(&mut self, from: &Operand, to: &Operand) {
        unsafe {
            self.get_manager_mut().replace_operand(from, to);
        }
    }

    /// Returns the operands of current instruction.
    ///
    /// # Safety
    /// You should not use this function, because it may cause unknown errors.
    #[inline]
    unsafe fn get_operand_mut(&mut self) -> &mut Vec<Operand> {
        &mut self.get_manager_mut().operand
    }

    /// Gets the previous instruction of current instruction.
    /// If current instruction is the first instruction of the `BasicBlock`, it will return None.
    ///
    /// # Panics
    /// Please make sure the current instruction is in the `BasicBlock`, otherwise it will panic.
    #[inline]
    fn get_prev(&self) -> Option<InstPtr> {
        let prev = self.get_manager().prev.unwrap();
        if let InstType::Head = prev.get_type() {
            None
        } else {
            Some(prev)
        }
    }

    /// Sets the previous instruction of current instruction.
    ///
    /// # Safety
    /// You should not use this function, because it may cause unknown errors.
    #[inline]
    unsafe fn set_prev(&mut self, inst: InstPtr) {
        self.get_manager_mut().prev = Some(inst);
    }

    /// Gets the next instruction of current instruction.
    /// If current instruction is the last instruction of the `BasicBlock`, it will return None.
    ///
    /// # Panics
    /// Please make sure the current instruction is in the `BasicBlock`, otherwise it will panic.
    #[inline]
    fn get_next(&self) -> Option<InstPtr> {
        let next = self.get_manager().next.unwrap();
        if let InstType::Head = next.get_type() {
            None
        } else {
            Some(next)
        }
    }

    /// Sets the next instruction of current instruction.
    ///
    /// # Safety
    /// You should not use this function, because it may cause unknown errors.
    #[inline]
    unsafe fn set_next(&mut self, inst: InstPtr) {
        self.get_manager_mut().next = Some(inst);
    }

    /// Returns the value type of current instruction.
    #[inline]
    fn get_value_type(&self) -> ValueType {
        self.get_manager().value_type.clone()
    }

    /// Moves the current instruction out of the `BasicBlock`.
    /// Please ensure that after moving out and inserting the current instruction into another `BasicBlock`,
    /// the current instruction will not be used again.
    ///
    /// # Safety
    /// This operation is not safe, use other methods instead.
    /// For example: insert_before, insert_after and remove_self.
    ///
    /// # Panics
    /// Only checked the error of having a predecessor but no successor, in which case it will panic.
    /// But for the case of having a successor but no predecessor, it does not report an error.
    unsafe fn move_self(&mut self) {
        if let Some(mut prev) = self.get_manager().prev {
            let mut next = self.get_manager().next.unwrap_or_else(|| {
                panic!(
                    "move_self failed! inst {} has a prev ({}) but no next",
                    self.get_type(),
                    prev.get_type()
                )
            });
            prev.set_next(next);
            next.set_prev(prev);
        }

        let manager = self.get_manager_mut();
        manager.prev = None;
        manager.next = None;
        manager.parent_bb = None;
    }

    /// Inserts a new instruction before the current instruction.
    /// The operation will first remove the new instruction from the original `BasicBlock`
    /// and then insert it into the specified position of the current `BasicBlock`.
    ///
    /// # Panics
    /// You need to ensure that the current instruction is definitely in the `BasicBlock`,
    /// otherwise it will panic.
    fn insert_before(&mut self, mut inst: InstPtr) {
        unsafe {
            inst.move_self();
            inst.set_parent_bb(self.get_parent_bb().unwrap())
        }

        let mut prev = self.get_manager().prev.unwrap();

        // 无法通过self获得指向自己的InstPtr，只有通过这种丑陋的方法了
        let mut self_ptr = prev.get_manager().next.unwrap();

        unsafe {
            prev.set_next(inst);
            self_ptr.set_prev(inst);
            inst.set_prev(prev);
            inst.set_next(self_ptr);
        }
    }

    /// Inserts a new instruction after the current instruction.
    /// The operation will first remove the new instruction from the original `BasicBlock`
    /// and then insert it into the specified position of the current `BasicBlock`.
    ///
    /// # Panics
    /// You need to ensure that the current instruction is definitely in the `BasicBlock`,
    /// otherwise it will panic.
    fn insert_after(&mut self, mut inst: InstPtr) {
        unsafe {
            inst.move_self();
            inst.set_parent_bb(self.get_parent_bb().unwrap());
        }

        unsafe {
            let mut next = self.get_manager_mut().next.unwrap();
            let mut self_ptr = next.get_manager_mut().prev.unwrap();
            next.set_prev(inst);
            self_ptr.set_next(inst);
            inst.set_prev(self_ptr);
            inst.set_next(next);
        }
    }

    /// Remove current instruction from the `BasicBlock`.
    /// This operation will remove the current instruction from the `BasicBlock` and clear the current operand.
    /// # Panics
    /// Same to move_self
    fn remove_self(&mut self) {
        unsafe {
            let id = self.get_id();
            self.move_self();

            let manager = self.get_manager_mut();

            manager.prev = None;
            manager.next = None;
            manager.operand.iter_mut().for_each(|op| match op {
                Operand::Instruction(inst) => {
                    inst.get_user_mut().retain(|user| user.get_id() != id);
                }
                Operand::Global(gl) => {
                    gl.get_user_mut().retain(|user| user.get_id() != id);
                }
                Operand::Parameter(par) => {
                    par.get_user_mut().retain(|user| user.get_id() != id);
                }
                Operand::Constant(_) => {}
            });
            manager.operand.clear();
        }
    }

    /// Replace current instruction with an operand.
    /// This operation will call `remove_self`, but update all users to use the new operand.
    fn replace_self(&mut self, operand: &Operand) {
        let user = self.get_user().to_vec();
        let self_operand = Operand::Instruction(self.get_manager().self_ptr.unwrap());
        user.into_iter().for_each(|mut user| {
            let operand_index = user
                .get_operand()
                .iter()
                .position(|op| op == &self_operand)
                .unwrap();
            user.set_operand(operand_index, operand.clone());
        });
        self.remove_self();
    }

    /// Returns the `BasicBlock` that current instruction belongs to.
    #[inline]
    fn get_parent_bb(&self) -> Option<BBPtr> {
        self.get_manager().parent_bb
    }

    /// Set the parent `BasicBlock` of current instruction.
    ///
    /// # Safety
    /// You should not use this function, because it may cause unknown errors.
    #[inline]
    unsafe fn set_parent_bb(&mut self, bb: BBPtr) {
        self.get_manager_mut().parent_bb = Some(bb);
    }

    /// Returns `True` if the current instruction is the last instruction in the `BasicBlock`.
    ///
    /// # Panics
    /// Please make sure the current instruction is in the `BasicBlock`, otherwise it will panic.
    #[inline]
    fn is_last(&self) -> bool {
        self.get_manager().next.unwrap().get_type() == InstType::Head
    }

    /// Returns `True` if the current instruction is the first instruction in the `BasicBlock`.
    ///
    /// # Panics
    /// Please make sure the current instruction is in the `BasicBlock`, otherwise it will panic.
    #[inline]
    fn is_first(&self) -> bool {
        self.get_manager().prev.unwrap().get_type() == InstType::Head
    }

    /// Returns the unique id of current instruction.
    fn get_id(&self) -> usize {
        self.get_manager().id.unwrap()
    }

    /// 将其生成相关的llvm ir
    fn gen_llvm_ir(&self) -> String;
}

impl PartialEq for dyn Instruction {
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

impl Eq for dyn Instruction {}

impl PartialOrd for dyn Instruction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.get_id().cmp(&other.get_id()))
    }
}

impl Ord for dyn Instruction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get_id().cmp(&other.get_id())
    }
}

/// Downcasts a `dyn instruction` to a `&T`, where `T` is a concrete `Instruction` type.
///
/// # Example
/// ```
/// # use compiler::middle::ir::instruction::{head::Head,downcast_ref};
/// # use compiler::middle::ir::ir_builder::IRBuilder;
/// # let mut ir_builder = IRBuilder::new();
/// let dyn_head = ir_builder.new_head();
/// let head = downcast_ref::<Head>(dyn_head.as_ref().as_ref());
/// ```
///
/// # Panics
/// If the downcast fails, this function will panic.
pub fn downcast_ref<T>(inst: &dyn Instruction) -> &T
where
    T: 'static + Instruction,
{
    unsafe {
        inst.as_any().downcast_ref::<T>().unwrap_or_else(|| {
            panic!(
                "downcast_ref failed! Try to get {} from {}",
                std::any::type_name::<T>(),
                inst.get_type()
            )
        })
    }
}

/// Downcasts a `dyn instruction` to a `&mut T`, where `T` is a concrete `Instruction` type.
///
/// # Example
/// ```
/// # use compiler::middle::ir::instruction::{head::Head,downcast_mut};
/// # use compiler::middle::ir::ir_builder::IRBuilder;
/// # let mut ir_builder = IRBuilder::new();
/// let mut dyn_head = ir_builder.new_head();
/// let add_inst = downcast_mut::<Head>(dyn_head.as_mut().as_mut());
/// ```
///
/// # Panics
/// If the downcast fails, this function will panic.
pub fn downcast_mut<T>(inst: &mut dyn Instruction) -> &mut T
where
    T: 'static + Instruction,
{
    let inst_type = inst.get_type();
    unsafe {
        inst.as_any_mut().downcast_mut::<T>().unwrap_or_else(|| {
            panic!(
                "downcast_mut failed! Try to get {} from {}",
                std::any::type_name::<T>(),
                inst_type
            )
        })
    }
}

/// Instruction Manager
/// This struct is used to manage the instructions.
/// Including def-use relationship, the relationship between instructions, etc.
pub struct InstManager {
    /// The unique id of current instruction.
    id: Option<usize>,
    /// The instructions that use current instruction as operand.
    /// For example: `add a, b`
    /// `a` and `b` are the operand of `add` instruction.
    /// At this time, the `user` of `a` and `b` both have `add` instruction.
    /// The order of `user` does not need to be considered.
    user: Vec<InstPtr>,

    /// The operand of current instruction.
    /// For example: `add a, b`
    /// At this time, the `add` instruction's operand has `a` and `b`.
    /// The order of `operand` needs to be considered.
    operand: Vec<Operand>,

    /// Prev instruction of current instruction, if current instruction is not in a `BasicBlock`, it is None.
    prev: Option<InstPtr>,

    /// Next instruction of current instruction, if current instruction is not in a `BasicBlock`, it is None.
    next: Option<InstPtr>,

    /// The `BasicBlock` that current instruction belongs to.
    parent_bb: Option<BBPtr>,

    /// Value type of current instruction.
    /// Default type is Void.
    value_type: ValueType,

    /// The ObjPtr of current instruction.
    self_ptr: Option<InstPtr>,
}

impl InstManager {
    pub fn new(value_type: ValueType) -> Self {
        Self {
            id: None,
            user: vec![],
            operand: vec![],
            prev: None,
            next: None,
            parent_bb: None,
            value_type,
            self_ptr: None,
        }
    }
}

impl InstManager {
    /// # Safety
    ///
    /// FIXME: explain why it is unsafe,and describe the safety requirements
    pub unsafe fn set_operand(&mut self, index: usize, new_op: Operand) {
        let old_op = std::mem::replace(&mut self.operand[index], new_op.clone());
        match old_op {
            Operand::Instruction(mut inst) => {
                inst.get_user()
                    .iter()
                    .position(|x| x.get_id() == self.id.unwrap())
                    .map(|index| inst.get_user_mut().remove(index));
            }
            Operand::Parameter(mut param) => {
                param
                    .get_user()
                    .iter()
                    .position(|x| x.get_id() == self.id.unwrap())
                    .map(|index| param.get_user_mut().remove(index));
            }
            Operand::Global(mut global) => {
                global
                    .get_user()
                    .iter()
                    .position(|x| x.get_id() == self.id.unwrap())
                    .map(|index| global.get_user_mut().remove(index));
            }
            _ => {}
        }
        match new_op {
            Operand::Instruction(mut inst) => {
                inst.get_user_mut().push(self.self_ptr.unwrap());
            }
            Operand::Parameter(mut param) => {
                param.add_user(self.self_ptr.unwrap());
            }
            Operand::Global(mut global) => {
                global.add_user(self.self_ptr.unwrap());
            }
            _ => {}
        }
    }

    /// # Safety
    ///
    /// FIXME: explain why it is unsafe,and describe the safety requirements
    pub unsafe fn replace_operand(&mut self, from: &Operand, to: &Operand) {
        match from {
            Operand::Instruction(mut inst) => {
                inst.get_user_mut().retain(|x| x != &self.self_ptr.unwrap());
            }
            Operand::Parameter(mut param) => {
                param
                    .get_user_mut()
                    .retain(|x| x != &self.self_ptr.unwrap());
            }
            Operand::Global(mut global) => {
                global
                    .get_user_mut()
                    .retain(|x| x != &self.self_ptr.unwrap());
            }
            _ => {}
        }
        match to {
            Operand::Instruction(mut inst) => {
                inst.get_user_mut().push(self.self_ptr.unwrap());
            }
            Operand::Parameter(mut param) => {
                param.add_user(self.self_ptr.unwrap());
            }
            Operand::Global(mut global) => {
                global.add_user(self.self_ptr.unwrap());
            }
            _ => {}
        }
        self.operand.iter_mut().for_each(|op| {
            if op == from {
                *op = to.clone();
            }
        });
    }

    /// # Safety
    ///
    /// FIXME: explain why it is unsafe,and describe the safety requirements
    pub unsafe fn add_operand(&mut self, operand: Operand) {
        match operand {
            Operand::Instruction(mut inst) => {
                inst.get_user_mut().push(self.self_ptr.unwrap());
            }
            Operand::Parameter(mut param) => {
                param.add_user(self.self_ptr.unwrap());
            }
            Operand::Global(mut global) => {
                global.add_user(self.self_ptr.unwrap());
            }
            _ => {}
        }
        self.operand.push(operand);
    }

    /// # Safety
    ///
    /// FIXME: explain why it is unsafe,and describe the safety requirements
    pub unsafe fn remove_operand(&mut self, index: usize) {
        let operand = self.operand.remove(index);
        match operand {
            Operand::Instruction(mut inst) => {
                inst.get_user()
                    .iter()
                    .enumerate()
                    .find_map(|(index, x)| {
                        if x.get_id() == self.id.unwrap() {
                            Some(index)
                        } else {
                            None
                        }
                    })
                    .map(|index| Some(inst.get_user_mut().remove(index)));
            }
            Operand::Parameter(mut param) => {
                param
                    .get_user()
                    .iter()
                    .enumerate()
                    .find_map(|(index, x)| {
                        if x.get_id() == self.id.unwrap() {
                            Some(index)
                        } else {
                            None
                        }
                    })
                    .map(|index| Some(param.get_user_mut().remove(index)));
            }
            Operand::Global(mut global) => {
                global
                    .get_user()
                    .iter()
                    .enumerate()
                    .find_map(|(index, x)| {
                        if x.get_id() == self.id.unwrap() {
                            Some(index)
                        } else {
                            None
                        }
                    })
                    .map(|index| Some(global.get_user_mut().remove(index)));
            }
            _ => {}
        }
    }

    /// # Safety
    ///
    /// FIXME: explain why it is unsafe,and describe the safety requirements
    pub unsafe fn set_id(&mut self, id: usize) {
        self.id = Some(id);
    }

    /// # Safety
    ///
    /// FIXME: explain why it is unsafe,and describe the safety requirements
    pub unsafe fn set_self_ptr(&mut self, self_ptr: InstPtr) {
        self.self_ptr = Some(self_ptr);
    }
}
