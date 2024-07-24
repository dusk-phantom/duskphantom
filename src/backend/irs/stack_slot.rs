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
#[derive(Debug)]
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
