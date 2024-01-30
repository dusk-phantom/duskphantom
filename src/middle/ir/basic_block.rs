use self::prog_mem_pool::ProgramMemPool;

use super::*;

pub type BBPtr = ObjPtr<BasicBlock>;

/// 基本块
/// 基本块主要结构为基本块名、基本块首指令
/// 基本块内部的指令格式为链表结构，最后一条指令必为跳转指令或者函数返回指令
pub struct BasicBlock {
    ///
    pub mem_pool: ObjPtr<ProgramMemPool>,
    /// 基本块名
    pub name: String,

    /// 基本块头指令，统一插入操作，无实际意义
    /// 逻辑上基本块的结构为双向的非循环链表，但在实际实现时为双向循环链表
    head_inst: InstPtr,

    /// 基本块的前驱基本块
    /// 前驱基本块的数量理论上可以为0到正无穷
    /// 当前驱基本块的数量为0时，该基本块为函数入口基本块，或者不可达基本块
    pred_bbs: Vec<BBPtr>,

    /// 基本块的后继基本块
    /// 后继基本块的数量理论上可以为0、1和2:
    /// 1. 当后继基本块的数量为0时，该基本块为函数出口基本块。
    /// 2. 当后继基本块的数量为1时，该基本块的最后一条指令为无条件跳转指令。
    /// 3. 当后继基本块的数量为2时，该基本块的最后一条指令为条件跳转指令。
    ///     + 下标为0的基本块为条件为真时跳转的基本块
    ///     + 下标为1的基本块为条件为假时跳转的基本块
    succ_bbs: Vec<BBPtr>,
}

impl BasicBlock {
    pub fn new(name: String, mem_pool: ObjPtr<ProgramMemPool>) -> Self {
        let head_inst = mem_pool
            .clone()
            .alloc_instruction(Box::new(instruction::head::Head::new()));
        Self {
            mem_pool,
            name,
            head_inst,
            pred_bbs: Vec::new(),
            succ_bbs: Vec::new(),
        }
    }

    /// 在申请到基本块的内存后初始化基本块
    /// 这是在设计上折中的丑陋代码，你不应该调用这个函数
    pub fn init_bb(mut bb: BBPtr) {
        unsafe {
            let mut head = bb.head_inst;
            bb.head_inst.set_prev(head);
            bb.head_inst.set_next(head);
            head.set_parent_bb(bb);
        }
    }

    /// 判断基本块是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.head_inst.is_last()
    }

    /// 获取基本块中的第一条指令
    ///
    /// # Panics
    /// 请先确保当前基本块不为空，否则会panic
    #[inline]
    pub fn get_first_inst(&self) -> InstPtr {
        self.head_inst.get_next().unwrap()
    }

    /// 获取基本块中的最后一条指令
    ///
    /// # Panics
    /// 请先确保当前基本块不为空，否则会panic
    #[inline]
    pub fn get_last_inst(&self) -> InstPtr {
        self.head_inst.get_prev().unwrap()
    }

    /// 在基本块最后插入一条指令
    #[inline]
    pub fn push_back(&mut self, inst: InstPtr) {
        self.head_inst.insert_before(inst);
    }

    /// 在基本块最前插入一条指令
    #[inline]
    pub fn push_front(&mut self, inst: InstPtr) {
        self.head_inst.insert_after(inst);
    }

    /// 判断是否为函数入口基本块
    #[inline]
    pub fn is_entry(&self) -> bool {
        self.pred_bbs.is_empty()
    }

    /// 判断是否为函数出口基本块
    #[inline]
    pub fn is_exit(&self) -> bool {
        self.succ_bbs.is_empty()
    }

    /// 获取前驱基本块
    #[inline]
    pub fn get_pred_bbs(&self) -> &Vec<BBPtr> {
        &self.pred_bbs
    }

    /// 获取后继基本块
    #[inline]
    pub fn get_succ_bbs(&self) -> &Vec<BBPtr> {
        &self.succ_bbs
    }

    /// 设置条件为真时跳转的基本块
    pub fn set_true_bb(&mut self, mut bb: BBPtr) {
        let self_ptr = ObjPtr::new(self);
        if self.succ_bbs.len() == 0 {
            self.succ_bbs.push(bb);
        } else {
            let mut next = self.succ_bbs[0];
            next.pred_bbs.retain(|x| *x != self_ptr);
            self.succ_bbs[0] = bb;
        }
        bb.pred_bbs.push(self_ptr);
    }

    /// 设置条件为假时跳转的基本块
    ///
    /// # Panics
    /// 需要先设置条件为真时跳转的基本块，否则会panic
    pub fn set_false_bb(&mut self, mut bb: BBPtr) {
        let self_ptr = ObjPtr::new(self);
        if self.succ_bbs.len() == 1 {
            self.succ_bbs.push(bb);
        } else {
            let mut next = self.succ_bbs[1];
            next.pred_bbs.retain(|x| *x != self_ptr);
            self.succ_bbs[1] = bb;
        }
        bb.pred_bbs.push(self_ptr);
    }

    pub fn iter(&self) -> BasicBlockIterator {
        BasicBlockIterator {
            cur: self.head_inst,
            next: self.head_inst.get_next(),
        }
    }
}

/// 基本块的迭代器，用于遍历基本块中的指令
pub struct BasicBlockIterator {
    cur: InstPtr,
    next: Option<InstPtr>,
}

impl Iterator for BasicBlockIterator {
    type Item = InstPtr;
    fn next(&mut self) -> Option<Self::Item> {
        self.cur = self.next?;
        self.next = self.cur.get_next();
        Some(self.cur)
    }
}
