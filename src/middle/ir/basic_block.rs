use super::*;

pub type BBPtr = ObjPtr<BasicBlock>;

/// 基本块
/// 基本块主要结构为基本块名、基本块首指令
/// 基本块内部的指令格式为链表结构，最后一条指令必为跳转指令或者函数返回指令
pub struct BasicBlock {
    /// 基本块名
    name: String,

    /// 基本块头指令，统一插入操作，无实际意义
    /// 逻辑上基本块的结构为双向的非循环链表，但在实际实现时为双向循环链表
    head_inst: instruction::head::Head,
}

impl BasicBlock {
    pub fn new(name: String) -> Self {
        Self {
            name,
            head_inst: instruction::head::Head::new(),
        }
    }

    /// 获取基本块名
    pub fn get_name(&self) -> &String {
        &self.name
    }

    /// 设置基本块名
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// 判断基本块是否为空
    pub fn is_empty(&self) -> bool {
        todo!()
    }

    /// 获取基本块中的头指令
    pub fn get_head_inst(&self) -> InstPtr {
        todo!()
    }

    /// 获取基本块中的尾指令
    pub fn get_tail_inst(&self) -> InstPtr {
        todo!()
    }

    /// 在基本块最后插入一条指令
    pub fn push_back(&mut self, inst: InstPtr) {
        todo!()
    }

    /// 在基本块最前插入一条指令
    pub fn push_front(&mut self, inst: InstPtr) {
        todo!()
    }
}
