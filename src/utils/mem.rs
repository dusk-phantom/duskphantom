use std::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

/// 裸指针，基于unsafe实现
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ObjPtr<T>(NonNull<T>);

impl<T> Deref for ObjPtr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T> DerefMut for ObjPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

pub struct ObjPool<T> {
    arena: typed_arena::Arena<std::pin::Pin<Box<T>>>,
}

impl<T> ObjPool<T> {
    /// 构造一个新的内存分配器
    pub fn new() -> Self {
        Self {
            arena: typed_arena::Arena::new(),
        }
    }

    /// 分配一块内存存放obj
    pub fn alloc(&mut self, obj: T) -> ObjPtr<T> {
        ObjPtr(NonNull::new(self.arena.alloc(Box::pin(obj)) as *const _ as *mut _).unwrap())
    }
}
