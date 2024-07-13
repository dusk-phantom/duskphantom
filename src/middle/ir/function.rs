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
    pub params: Vec<ParaPtr>,
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

    pub fn gen_llvm_ir(&self) -> String {
        let mut ir = format!("define {} @{}(", self.return_type, self.name);
        if !self.params.is_empty() {
            for param in self.params.iter() {
                ir += &format!("{}, ", param.as_ref());
            }
            let _ = ir.split_off(ir.len() - 2);
        }
        ir += ") {\n";
        self.bfs_iter().for_each(|bb| {
            ir += &bb.gen_llvm_ir();
        });
        ir + "\n}\n"
    }
}

define_graph_iterator!(BFSIterator, VecDeque<BBPtr>, pop_front, get_succ_bb);
define_graph_iterator!(BFSIteratorRev, VecDeque<BBPtr>, pop_front, get_pred_bb);
define_graph_iterator!(DFSIterator, Vec<BBPtr>, pop, get_succ_bb);
define_graph_iterator!(DFSIteratorRev, Vec<BBPtr>, pop, get_pred_bb);

pub type ParaPtr = ObjPtr<Parameter>;
impl Display for ParaPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{}", self.name)
    }
}

#[derive(Clone)]
pub struct Parameter {
    pub name: String,
    pub value_type: ValueType,
    user: Vec<InstPtr>,
}

impl Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} %{}", self.value_type, self.name)
    }
}

impl Parameter {
    pub fn new(name: String, value_type: ValueType) -> Self {
        Self {
            name,
            value_type,
            user: Vec::new(),
        }
    }

    pub fn get_user(&self) -> &[InstPtr] {
        &self.user
    }
    pub fn get_user_mut(&mut self) -> &mut Vec<InstPtr> {
        &mut self.user
    }
    /// # Safety
    /// FIXME: explain why it is unsafe,and describe the safety requirements
    pub unsafe fn add_user(&mut self, inst: InstPtr) {
        self.user.push(inst);
    }
    /// # Safety
    /// FIXME: explain why it is unsafe,and describe the safety requirements
    pub unsafe fn remove_user(&mut self, inst: InstPtr) {
        self.user
            .iter()
            .position(|x| *x == inst)
            .map(|i| self.user.swap_remove(i));
    }
}
