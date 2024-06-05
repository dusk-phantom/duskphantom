//内存数据结构, 维护内存段,包含上下限,隐含大小
pub struct StackSlot {
    start: usize,
    end: usize,
}
impl StackSlot {
    pub fn size(&self) -> usize {
        self.end - self.start
    }
}

// 内存分配器,记录栈顶位置
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
            end: self.alloc_from + size,
        };
        self.alloc_from += size;
        ret
    }
}
