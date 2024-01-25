//高并发id分配器,用于分配虚拟寄存器使用的id
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
// 能够高并发地管理一定 数据范围内的id分配
#[derive(Clone)]
pub struct ParalCounter {
    end: usize,
    counter: Arc<AtomicUsize>,
}

impl ParalCounter {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            end,
            counter: Arc::new(AtomicUsize::new(start)),
        }
    }
    pub fn get_id(&self) -> Option<usize> {
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        if id < self.end {
            Some(id)
        } else {
            None
        }
    }
}
