use self::prog_mem_pool::ProgramMemPool;

use super::*;
use crate::define_graph_iterator;

pub type FunPtr = ObjPtr<Function>;

/// 函数
pub struct Function {
    /// mem_pool
    pub mem_pool: ObjPtr<ProgramMemPool>,

    /// 函数名
    pub name: String,

    /// 函数入口，若为库函数，则为None
    pub entry: Option<BBPtr>,

    /// 函数出口，若为库函数，则为None
    pub exit: Option<BBPtr>,

    /// 返回值类型
    pub return_type: ValueType,

    /// 函数参数，存放在一个基本块中
    /// 便于统一指令的运算
    pub params: BBPtr,
}

impl Function {
    /// 构造一个空函数
    pub fn new(name: String, return_type: ValueType, mem_pool: ObjPtr<ProgramMemPool>) -> Self {
        let params = mem_pool
            .clone()
            .alloc_basic_block(BasicBlock::new("params".to_string(), mem_pool));
        Self {
            mem_pool,
            name,
            entry: None,
            exit: None,
            return_type,
            params,
        }
    }

    /// 检查是否为库函数
    pub fn is_lib(&self) -> bool {
        self.entry.is_none()
    }

    /// 创建一个深度优先迭代器，用以深度优先遍历基本块的图结构。
    /// 以函数入口为起点，顺数据流方向遍历
    /// 在遍历过程中请勿更改图结构，这可能会导致未知的错误
    pub fn dfs_iter(&self) -> DFSIterator {
        DFSIterator::from(self.entry.unwrap())
    }

    /// 创建一个广度优先迭代器，用以广度优先遍历基本块的图结构
    /// 以函数入口为起点，顺数据流方向遍历
    /// 在遍历过程中请勿更改图结构，这可能会导致未知的错误
    pub fn bfs_iter(&self) -> BFSIterator {
        BFSIterator::from(self.entry.unwrap())
    }

    /// 创建一个深度优先迭代器，用以深度优先遍历基本块的图结构
    /// 以函数出口为起点，逆数据流方向遍历
    /// 在遍历过程中请勿更改图结构，这可能会导致未知的错误
    pub fn dfs_iter_rev(&self) -> DFSIteratorRev {
        DFSIteratorRev::from(self.exit.unwrap())
    }

    /// 创建一个广度优先迭代器，用以广度优先遍历基本块的图结构
    /// 以函数出口为起点，逆数据流方向遍历
    /// 在遍历过程中请勿更改图结构，这可能会导致未知的错误
    pub fn bfs_iter_rev(&self) -> BFSIteratorRev {
        BFSIteratorRev::from(self.exit.unwrap())
    }
}

define_graph_iterator!(BFSIterator, VecDeque<BBPtr>, pop_front, get_succ_bbs);
define_graph_iterator!(BFSIteratorRev, VecDeque<BBPtr>, pop_front, get_pred_bbs);
define_graph_iterator!(DFSIterator, Vec<BBPtr>, pop, get_succ_bbs);
define_graph_iterator!(DFSIteratorRev, Vec<BBPtr>, pop, get_pred_bbs);
