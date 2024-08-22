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

use std::{
    fmt::Debug,
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

/// Deterministic ordering for ObjPtr.
/// Comparing by pointer address is non-deterministic, so we compare by the object itself.
impl<T> PartialOrd for ObjPtr<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<T> Ord for ObjPtr<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

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

impl<T> Debug for ObjPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:p}", self.as_ref())
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
