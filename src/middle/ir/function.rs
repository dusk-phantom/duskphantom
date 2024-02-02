use super::*;
use crate::define_graph_iterator;

pub type FunPtr = ObjPtr<Function>;

pub struct Function {
    pub mem_pool: ObjPtr<IRBuilder>,

    pub name: String,

    /// Entry of function, if it is a function that is not defined in this module, it will be None.
    /// Such as library function.
    pub entry: Option<BBPtr>,

    /// Exit of function, if it is a function that is not defined in this module, it will be None.
    /// Such as library function.
    pub exit: Option<BBPtr>,

    pub return_type: ValueType,

    /// BasicBlock of function parameters
    pub params: BBPtr,
}

impl Function {
    /// return True if it is a function that is not defined in this module.
    pub fn is_lib(&self) -> bool {
        self.entry.is_none()
    }

    /// Create a depth-first iterator to traverse the graph structure of basicblocks.
    /// Traverse in the direction of data flow with the function entry as the starting point.
    /// Do not change the graph structure during traversal, which may cause unknown errors
    pub fn dfs_iter(&self) -> DFSIterator {
        DFSIterator::from(self.entry.unwrap())
    }

    /// Create a breadth-first iterator to traverse the graph structure of basicblocks.
    /// Traverse in the direction of data flow with the function entry as the starting point.
    /// Do not change the graph structure during traversal, which may cause unknown errors
    pub fn bfs_iter(&self) -> BFSIterator {
        BFSIterator::from(self.entry.unwrap())
    }

    /// Create a depth-first iterator to traverse the graph structure of basicblocks.
    /// Traverse in the reverse direction of data flow with the function exit as the starting point.
    /// Do not change the graph structure during traversal, which may cause unknown errors
    pub fn dfs_iter_rev(&self) -> DFSIteratorRev {
        DFSIteratorRev::from(self.exit.unwrap())
    }

    /// Create a breadth-first iterator to traverse the graph structure of basicblocks.
    /// Traverse in the reverse direction of data flow with the function exit as the starting point.
    /// Do not change the graph structure during traversal, which may cause unknown errors
    pub fn bfs_iter_rev(&self) -> BFSIteratorRev {
        BFSIteratorRev::from(self.exit.unwrap())
    }
}

define_graph_iterator!(BFSIterator, VecDeque<BBPtr>, pop_front, get_succ_bb);
define_graph_iterator!(BFSIteratorRev, VecDeque<BBPtr>, pop_front, get_pred_bb);
define_graph_iterator!(DFSIterator, Vec<BBPtr>, pop, get_succ_bb);
define_graph_iterator!(DFSIteratorRev, Vec<BBPtr>, pop, get_pred_bb);
