//高并发id分配器,用于分配虚拟寄存器使用的id
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
// 能够高并发地管理一定 数据范围内的id分配
#[derive(Clone)]
pub struct ParalCounter {
    start: usize,
    end: usize,
    counter: Arc<AtomicUsize>,
}
