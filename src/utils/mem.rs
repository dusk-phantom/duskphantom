use std::{
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

/// 裸指针，基于unsafe实现
pub struct ObjPtr<T>(NonNull<T>);

impl<T> ObjPtr<T> {
    pub fn new(ptr: &T) -> ObjPtr<T> {
        ObjPtr(NonNull::new(ptr as *const _ as *mut _).unwrap())
    }
}
impl<T> AsRef<T> for ObjPtr<T> {
    fn as_ref(&self) -> &T {
        unsafe { self.0.as_ref() }
    }
}
impl<T> AsMut<T> for ObjPtr<T> {
    fn as_mut(&mut self) -> &mut T {
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
        *self
    }
}

impl<T> Copy for ObjPtr<T> {}

impl<T> PartialEq for ObjPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.as_ref(), other.as_ref())
    }
}

impl<T> Eq for ObjPtr<T> {}

impl<T> Hash for ObjPtr<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.as_ref(), state)
    }
}

impl<T> From<&T> for ObjPtr<T> {
    fn from(ptr: &T) -> Self {
        Self::new(ptr)
    }
}

impl<T> From<&mut T> for ObjPtr<T> {
    fn from(ptr: &mut T) -> Self {
        Self::new(ptr)
    }
}

pub struct ObjPool<T>
where
    T: Sized,
{
    arena: typed_arena::Arena<std::pin::Pin<Box<T>>>,
}

impl<T> Default for ObjPool<T>
where
    T: Sized,
{
    fn default() -> Self {
        Self {
            arena: typed_arena::Arena::new(),
        }
    }
}
impl<T> ObjPool<T> {
    /// 构造一个新的内存分配器
    pub fn new() -> Self {
        Default::default()
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
