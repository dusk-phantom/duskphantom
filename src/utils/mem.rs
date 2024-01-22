use std::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

/// 裸指针，基于unsafe实现
pub struct ObjPtr<T>(NonNull<T>);

impl<T> ObjPtr<T> {
    pub fn new(ptr: &T) -> ObjPtr<T> {
        ObjPtr(NonNull::new(ptr as *const _ as *mut _).unwrap())
    }

    pub fn as_ref(&self) -> &T {
        unsafe { self.0.as_ref() }
    }

    pub fn as_mut(&mut self) -> &mut T {
        unsafe { self.0.as_mut() }
    }
}

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

impl<T> Clone for ObjPtr<T> {
    fn clone(&self) -> Self {
        ObjPtr(self.0.clone())
    }
}

impl<T> Copy for ObjPtr<T> {}

impl<T> PartialEq for ObjPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.as_ref(), other.as_ref())
    }
}

impl<T> Eq for ObjPtr<T> {}

pub struct ObjPool<T>
where
    T: Sized,
{
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
        ObjPtr::new(self.arena.alloc(Box::pin(obj)))
    }

    /// 清空内存池
    pub fn clear(&mut self) {
        self.arena = typed_arena::Arena::new();
    }
}
