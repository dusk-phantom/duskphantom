pub mod basic_block;
mod context_arena;
pub mod function;
pub mod instruction;
pub mod module;

pub use self::basic_block::BasicBlock;
pub use self::function::Function;
pub use self::instruction::Instruction;
pub use self::module::Module;

use self::context_arena::ContextArena;
use generational_arena::{Arena, Index};
use std::{collections::HashMap, pin::Pin};

/// 表示函数指针
type FunPtr = Index;
/// 表示基本块指针
type BBPtr = Index;
/// 表示指令指针
type InstPtr = Index;

use std::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

#[derive(Clone, Copy)]
pub struct ObjPtr<T>(NonNull<T>);

impl<T> ObjPtr<T> {
    pub fn as_ref(&self) -> &T {
        unsafe { self.0.as_ref() }
    }

    pub fn as_mut(&mut self) -> &mut T {
        unsafe { self.0.as_mut() }
    }

    pub fn new(ptr: &T) -> Self {
        let ptr = NonNull::from(ptr);
        Self(ptr)
    }
}

impl<T> Deref for ObjPtr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for ObjPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}
