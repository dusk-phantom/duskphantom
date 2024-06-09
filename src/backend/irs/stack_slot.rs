/// StackSlot: represents a stack slot, which is a contiguous memory region on the stack.

#[derive(Debug, Clone)]
pub struct StackSlot {
    start: usize,
    size: usize,
}
impl StackSlot {
    /// get the start address of the stack slot,stack_slot[start]=<s> means
    /// this stack slot if from <s>(sp) to <e>(sp)
    pub fn start(&self) -> usize {
        self.start
    }
    pub fn end(&self) -> usize {
        self.start + self.size
    }
    pub fn size(&self) -> usize {
        self.size
    }
}

// StackAllocator: a simple stack allocator for stack slots.
pub struct StackAllocator {
    alloc_from: usize,
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
    pub fn alloc(&mut self, size: usize) -> StackSlot {
        let ret = StackSlot {
            start: self.alloc_from,
            size,
        };
        self.alloc_from += size;
        ret
    }
    pub fn allocated(&self) -> usize {
        self.alloc_from
    }
}
