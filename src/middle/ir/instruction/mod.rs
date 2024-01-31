use super::*;
pub mod head;

pub type InstPtr = ObjPtr<Box<dyn Instruction>>;

use crate::gen_common_code;
use std::any::Any;
/// 指令中有很多共有的方法，在此将其抽象为一个 trait
pub trait Instruction {
    /// # Safety
    /// 不要调用此方法，统一使用downcast_ref
    unsafe fn as_any(&self) -> &dyn Any;

    /// # Safety
    /// 不要调用此方法，统一使用downcast_mut
    unsafe fn as_any_mut(&mut self) -> &mut dyn Any;

    /// 获取当前指令的类型
    fn get_type(&self) -> InstType;

    /// 获取当前指令的管理器的不可变引用
    fn get_manager(&self) -> &InstManager;

    /// 获取当前指令的管理器的可变引用
    ///
    /// # Safety
    /// 你不应该使用这个函数，这可能会导致未知的错误
    unsafe fn get_manager_mut(&mut self) -> &mut InstManager;

    /// 获取使用当前指令作为操作数的指令
    #[inline]
    fn get_user(&self) -> &[InstPtr] {
        &self.get_manager().user
    }

    /// 获取使用当前指令作为操作数的指令的可变引用
    ///
    /// # Safety
    /// 你不应该使用这个函数，因为def-use需要双方共同维持，单方面修改会发生未知的错误
    #[inline]
    unsafe fn get_user_mut(&mut self) -> &mut Vec<InstPtr> {
        &mut self.get_manager_mut().user
    }

    /// 获取当前指令的操作数的引用
    #[inline]
    fn get_operand(&self) -> &[InstPtr] {
        &self.get_manager().operand
    }

    /// 获取当前指令的操作数的可变引用
    ///
    /// # Safety
    /// 你不应该使用这个函数，因为def-use需要双方共同维持，单方面修改会发生未知的错误
    #[inline]
    unsafe fn get_operand_mut(&mut self) -> &mut Vec<InstPtr> {
        &mut self.get_manager_mut().operand
    }

    /// 获取当前指令的前驱指令
    /// 若当前指令为第一条指令，则返回None
    ///
    /// # Panics
    /// 请确保当前指令在基本块中，否则会panic
    #[inline]
    fn get_prev(&self) -> Option<InstPtr> {
        let prev = self.get_manager().prev.unwrap();
        if let InstType::Head = prev.get_type() {
            None
        } else {
            Some(prev)
        }
    }

    /// 设置当前指令的前驱指令
    ///
    /// # Safety
    /// 你不应该使用这个函数，这可能会导致未知的错误
    #[inline]
    unsafe fn set_prev(&mut self, inst: InstPtr) {
        self.get_manager_mut().prev = Some(inst);
    }

    /// 获取当前指令的后继指令
    /// 若当前指令为最后一条指令，则返回None
    ///
    /// # Panics
    /// 请确保当前指令在基本块中，否则会panic
    #[inline]
    fn get_next(&self) -> Option<InstPtr> {
        let next = self.get_manager().next.unwrap();
        if let InstType::Head = next.get_type() {
            None
        } else {
            Some(next)
        }
    }

    /// 设置当前指令的后继指令
    ///
    /// # Safety
    /// 你不应该使用这个函数，这可能会导致未知的错误
    #[inline]
    unsafe fn set_next(&mut self, inst: InstPtr) {
        self.get_manager_mut().next = Some(inst);
    }

    /// 获取当前指令计算结果的类型
    #[inline]
    fn get_value_type(&self) -> ValueType {
        self.get_manager().value_type
    }

    /// 将指令移出当前基本块中，请保证在移出后和将当前指令插入别的
    /// 基本块前，没有任何对当前指令的的操作。
    ///
    /// # Safety
    /// 不建议使用此指令，请使用别的方法。例如：insert_before、insert_after以及remove_self
    ///
    /// # Panics
    /// 只检查了在有前驱的情况下没有后继的错误，此时会panic。
    /// 但是对于只有后继没有前驱的情况，没有报错
    unsafe fn move_self(&mut self) {
        if let Some(mut prev) = self.get_prev() {
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

    /// 在当前指令前插入一条指令inst，inst可以在原本就在基本块中。
    /// 该操作会先将其从原本的基本块中移出，然后再插入到当前的基本块的指定位置中。
    ///
    /// # Panics
    /// 需要保证当前指令一定在基本块中，否则会panic
    fn insert_before(&mut self, mut inst: InstPtr) {
        unsafe {
            inst.move_self();
            inst.set_parent_bb(self.get_parent_bb().unwrap())
        }

        let mut prev = self.get_prev().unwrap();

        // 无法通过self获得指向自己的InstPtr，只有通过这种丑陋的方法了
        let mut self_ptr = prev.get_next().unwrap();

        unsafe {
            prev.set_next(inst);
            self_ptr.set_prev(inst);
            inst.set_prev(prev);
            inst.set_next(self_ptr);
        }
    }

    /// 在当前指令后插入一条指令
    /// 该操作会先将其从原本的基本块中移出，然后再插入到当前的基本块的指定位置中。
    ///
    /// # Panics
    /// 需要保证当前指令一定在基本块中，否则会panic
    fn insert_after(&mut self, mut inst: InstPtr) {
        unsafe {
            inst.move_self();
            inst.set_parent_bb(self.get_parent_bb().unwrap());
        }

        let mut next = self.get_next().unwrap();

        unsafe {
            // 无法通过self获得指向自己的InstPtr，只有通过这种丑陋的方法了
            let mut self_ptr = next.get_prev().unwrap();
            next.set_prev(inst);
            self_ptr.set_next(inst);
            inst.set_prev(self_ptr);
            inst.set_next(next);
        }
    }

    /// 将当前指令从基本块中移出。
    /// 该操作会将当前指令移出基本块中，并清除当前的操作数。
    fn remove_self(&mut self)
    where
        Self: Sized,
    {
        unsafe {
            self.move_self();

            // 丑陋代码，不知道该怎么优化
            let self_p = ObjPtr::new(self);

            let manager = self.get_manager_mut();

            manager.prev = None;
            manager.next = None;
            manager.operand.iter_mut().for_each(|op| {
                op.get_user_mut()
                    .retain(|user| !is_same(user.as_ref().as_ref(), self_p.as_ref()));
            });
            manager.operand.clear();
        }
    }

    /// 获得当前指令所在的基本块
    #[inline]
    fn get_parent_bb(&self) -> Option<BBPtr> {
        self.get_manager().parent_bb
    }

    /// 设置当前指令所在的基本块
    ///
    /// # Safety
    /// 你不应该使用这个函数，这可能会导致未知的错误
    #[inline]
    unsafe fn set_parent_bb(&mut self, bb: BBPtr) {
        self.get_manager_mut().parent_bb = Some(bb);
    }

    /// 判断是否为最后一条指令
    ///
    /// # Panics
    /// 请先确保当前指令在基本块中，否则会panic
    #[inline]
    fn is_last(&self) -> bool {
        self.get_manager().next.unwrap().get_type() == InstType::Head
    }

    /// 判断是否为第一条指令
    ///
    /// # Panics
    /// 请先确保当前指令在基本块中，否则会panic
    #[inline]
    fn is_first(&self) -> bool {
        self.get_manager().prev.unwrap().get_type() == InstType::Head
    }

    /// 将其生成相关的llvm ir
    fn gen_llvm_ir(&self) -> String;
}

/// 用于实现Instruction中的通用方法，简化代码
/// 需要保证指令内部存在InstManager，且命名为manager
#[macro_export]
macro_rules! gen_common_code {
    ($type:ty,$id:ident) => {
        #[inline]
        unsafe fn as_any(&self) -> &dyn Any {
            self
        }
        #[inline]
        unsafe fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
        #[inline]
        fn get_type(&self) -> InstType {
            InstType::$id
        }
        #[inline]
        fn get_manager(&self) -> &InstManager {
            &self.manager
        }
        #[inline]
        unsafe fn get_manager_mut(&mut self) -> &mut InstManager {
            &mut self.manager
        }
    };
}

/// 当需要使用具体类型的指令方法时，使用此方法来获取具体指令类型的引用
/// 使用前请先确保当前指令为指定类型
///
/// # Example
/// ```
/// # use compiler::middle::ir::InstPtr;
/// # use compiler::middle::ir::instruction::{head::Head,Instruction,downcast_ref};
/// let dyn_head: Box<dyn Instruction> = Box::new(Head::new());
/// let head = downcast_ref::<Head>(dyn_head.as_ref());
/// ```
///
/// # Panics
/// 若当前指令不为指定类型时，会panic!("downcast_ref failed")
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

/// 当需要使用具体类型的指令方法时，使用此方法来获取具体指令类型的可变引用
/// 使用前请先确保当前指令为指定类型
///
/// # Example
/// ```
/// # use compiler::middle::ir::InstPtr;
/// # use compiler::middle::ir::instruction::{head::Head,Instruction,downcast_mut};
/// let mut dyn_head: Box<dyn Instruction> = Box::new(Head::new());
/// let add_inst = downcast_mut::<Head>(dyn_head.as_mut());
/// ```
///
/// # Panics
/// 若当前指令不为指定类型时，会panic!("downcast_mut failed")
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

/// 判断两个&dyn Instruction是否为同一个指令的引用
#[inline]
pub fn is_same(left: &dyn Instruction, right: &dyn Instruction) -> bool {
    left as *const dyn Instruction == right as *const dyn Instruction
}

/// 指令类型
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstType {
    /// 无实际意义，平时操作不会拿到这个指令
    Head,
}

use std::fmt::{Display, Formatter};
impl Display for InstType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InstType::Head => write!(f, "InstType::Head"),
        }
    }
}

/// 管理指令的统一功能
/// 包括def-use关系、指令的前后关系等
pub struct InstManager {
    /// 使用当前指令作为操作数的指令，与operand互为逆关系
    /// 例如：add a, b
    /// 此时a和b的user中都有add指令
    /// user的顺序不需要考虑
    user: Vec<InstPtr>,

    /// 当前指令的操作数
    /// 例如：add a, b
    /// 此时add指令的operand中有a和b
    /// 要注意operand的顺序
    operand: Vec<InstPtr>,

    /// 当前指令的前驱指令，若当前指令不在基本块中，则为None
    prev: Option<InstPtr>,

    /// 当前指令的后继指令，若当前指令不在基本块中，则为None
    next: Option<InstPtr>,

    /// 当前指令所在的基本块
    parent_bb: Option<BBPtr>,

    /// 当前指令计算结果的类型
    /// 默认类型为Void
    value_type: ValueType,
}

impl InstManager {
    pub fn new() -> Self {
        Self {
            user: vec![],
            operand: vec![],
            prev: None,
            next: None,
            parent_bb: None,
            value_type: ValueType::Void,
        }
    }
}
