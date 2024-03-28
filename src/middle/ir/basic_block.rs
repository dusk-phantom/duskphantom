use super::*;

pub type BBPtr = ObjPtr<BasicBlock>;

/// The organization structure of the instructions inside the basic block is a double linked list.
/// the last instruction must be br or ret.
pub struct BasicBlock {
    pub ir_builder: ObjPtr<IRBuilder>,

    pub name: String,

    /// The head instruction of the `BasicBlock`,
    /// which is used to unify the insertion operation and has no actual meaning.
    /// Logical structure of the `BasicBlock` is a two-way non-circular linked list,
    /// but in actual implementation, it is a two-way circular linked list.
    head_inst: InstPtr,

    /// The predecessor `BasicBlock` of the `BasicBlock`.
    /// The number of predecessor `BasicBlocks` can theoretically be 0 to infinity.
    /// When the number of predecessor `BasicBlocks` is 0,
    /// the `BasicBlock` is the function entry `BasicBlock` or an unreachable `BasicBlock`.
    pred_bbs: Vec<BBPtr>,

    /// The successor `BasicBlock` of the `BasicBlock`.
    /// The number of successor `BasicBlocks` can theoretically be 0, 1, and 2:
    /// 1. When the number of successor `BasicBlocks` is 0, the `BasicBlock` is the function exit `BasicBlock`.
    /// 2. When the number of successor `BasicBlocks` is 1, the last instruction of the `BasicBlock` is an unconditional jump instruction.
    /// 3. When the number of successor `BasicBlocks` is 2, the last instruction of the `BasicBlock` is a conditional jump instruction.
    ///    + The `BasicBlock` with index 0 is the `BasicBlock` to jump to when the condition is true.
    ///    + The `BasicBlock` with index 1 is the `BasicBlock` to jump to when the condition is false.
    succ_bbs: Vec<BBPtr>,
}

impl BasicBlock {
    pub fn new(name: String, mut ir_builder: ObjPtr<IRBuilder>) -> Self {
        let head_inst = ir_builder.new_head();
        Self {
            ir_builder,
            name,
            head_inst,
            pred_bbs: Vec::new(),
            succ_bbs: Vec::new(),
        }
    }

    /// Inits `BasicBlock` after memory allocation.
    /// This is an ugly code that is a compromise in design. You should not call this function.
    pub unsafe fn init_bb(mut bb: BBPtr) {
        let mut head = bb.head_inst;
        bb.head_inst.set_prev(head);
        bb.head_inst.set_next(head);
        head.set_parent_bb(bb);
    }

    /// Returns `True` if the `BasicBlock` is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.head_inst.is_last()
    }

    /// Gets first instruction in the `BasicBlock`.
    ///
    /// # Panics
    /// Please make sure the current basic block is not empty, otherwise it will panic.
    #[inline]
    pub fn get_first_inst(&self) -> InstPtr {
        self.head_inst.get_next().unwrap()
    }

    /// Gets the last instruction in the `BasicBlock`.
    ///
    /// # Panics
    /// Please make sure the current basic block is not empty, otherwise it will panic.
    #[inline]
    pub fn get_last_inst(&self) -> InstPtr {
        self.head_inst.get_prev().unwrap()
    }

    /// Appends a new instruction to the end of the `BasicBlock`.
    #[inline]
    pub fn push_back(&mut self, inst: InstPtr) {
        self.head_inst.insert_before(inst);
    }

    /// Appends a new instruction to the beginning of the `BasicBlock`.
    #[inline]
    pub fn push_front(&mut self, inst: InstPtr) {
        self.head_inst.insert_after(inst);
    }

    /// Returns `True` if the `BasicBlock` is the function entry `BasicBlock`.
    #[inline]
    pub fn is_entry(&self) -> bool {
        self.pred_bbs.is_empty()
    }

    /// Returns `True` if the `BasicBlock` is the function exit `BasicBlock`.
    #[inline]
    pub fn is_exit(&self) -> bool {
        self.succ_bbs.is_empty()
    }

    /// Gets the predecessor `BasicBlock` of the `BasicBlock`.
    #[inline]
    pub fn get_pred_bb(&self) -> &Vec<BBPtr> {
        &self.pred_bbs
    }

    /// Gets the successor `BasicBlock` of the `BasicBlock`.
    #[inline]
    pub fn get_succ_bb(&self) -> &Vec<BBPtr> {
        &self.succ_bbs
    }

    /// Sets which `BasicBlock` to jump to when the condition is true.
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

    /// Sets which `BasicBlock` to jump to when the condition is false.
    ///
    /// # Panics
    /// You should set the true `BasicBlock` before use this method.
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

    /// Returns a iterator of the `BasicBlock`.
    /// The iterator yeilds the `InstPtr` of the `BasicBlock` except the head instruction.
    pub fn iter(&self) -> BasicBlockIterator {
        BasicBlockIterator {
            cur: self.head_inst,
            next: self.head_inst.get_next(),
        }
    }

    pub fn gen_llvm_ir(&self) -> String {
        todo!()
    }
}

impl Display for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{}", self.name)
    }
}

impl Extend<InstPtr> for BasicBlock {
    fn extend<T: IntoIterator<Item = InstPtr>>(&mut self, iter: T) {
        iter.into_iter().for_each(|inst| {
            self.push_back(inst);
        });
    }
}

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
