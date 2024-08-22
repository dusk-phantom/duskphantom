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

/// StackSlot: represents a stack slot, which is a contiguous memory region on the stack.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StackSlot {
    start: u32,
    size: u32,
}
impl StackSlot {
    /// get the start address of the stack slot,stack_slot[start]=<s> means
    /// this stack slot if from <s>(sp) to <e>(sp)
    pub fn start(&self) -> u32 {
        self.start
    }
    pub fn end(&self) -> u32 {
        self.start + self.size
    }
    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn gen_asm(&self) -> String {
        format!("[{}-{}]", self.start, self.end())
    }
}

// StackAllocator: a simple stack allocator for stack slots.
#[derive(Debug, Clone)]
pub struct StackAllocator {
    alloc_from: u32,
}
impl Default for StackAllocator {
    fn default() -> Self {
        StackAllocator::new()
    }
}
impl StackAllocator {
    pub fn new() -> StackAllocator {
        StackAllocator { alloc_from: 0 }
    }

    // alloc num byte size memory
    pub fn alloc(&mut self, num_byte: u32) -> StackSlot {
        let ret = StackSlot {
            start: self.alloc_from,
            size: num_byte,
        };
        self.alloc_from += num_byte;
        ret
    }

    // return how many byte had been allocated
    pub fn allocated(&self) -> u32 {
        self.alloc_from
    }
}
