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
            let mut next = self.get_next().unwrap_or_else(|| {
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

        let mut next = self.get_next().unwrap();

        unsafe {
            let mut self_ptr = next.get_prev().unwrap();
            next.set_prev(inst);
            self_ptr.set_next(inst);
            inst.set_prev(self_ptr);
            inst.set_next(next);
        }
    }

    /// Remove current instruction from the `BasicBlock`.
    /// This operation will remove the current instruction from the `BasicBlock` and clear the current operand.
    fn remove_self(&mut self)
    where
        Self: Sized,
    {
        unsafe {
            self.move_self();

            let self_p = ObjPtr::new(self);

            let manager = self.get_manager_mut();

            manager.prev = None;
            manager.next = None;
            manager.operand.iter_mut().for_each(|op| {
                if let Operand::Instruction(op) = op {
                    op.get_user_mut()
                        .retain(|user| !std::ptr::eq(user.as_ref().as_ref(), self_p.as_ref()));
                }
            });
            manager.operand.clear();
        }
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
    pub unsafe fn set_operand(&mut self, index: usize, mut operand: Operand) {
        if let Operand::Instruction(mut inst) = self.operand[index] {
            let user = inst.get_user_mut();
            user.iter()
                .position(|x| x == &self.self_ptr.unwrap())
                .map(|x| user.swap_remove(x));
        }
        self.operand[index] = operand;
    }

    pub unsafe fn add_operand(&mut self, mut operand: Operand) {
        match operand {
            Operand::Instruction(mut inst) => {
                inst.get_user_mut().push(self.self_ptr.unwrap());
            }
            Operand::Parametr(mut param) => {
                param.add_user(self.self_ptr.unwrap());
            }
            Operand::Global(mut global) => {
                global.add_user(self.self_ptr.unwrap());
            }
            _ => {}
        }
        self.operand.push(operand);
    }

    pub unsafe fn set_id(&mut self, id: usize) {
        self.id = Some(id);
    }

    pub unsafe fn set_self_ptr(&mut self, self_ptr: InstPtr) {
        self.self_ptr = Some(self_ptr);
    }
}
