use super::*;
/// 基本块
/// 基本块主要结构为基本块名、基本块首指令
/// 基本块内部的指令格式为链表结构，最后一条指令必为跳转指令或者函数返回指令
pub struct BasicBlock {
    name: String,
}

impl BasicBlock {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
