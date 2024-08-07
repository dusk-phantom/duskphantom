use std::collections::HashSet;

use crate::{
    backend::from_self::downcast_ref,
    middle::{
        ir::{
            instruction::{misc_inst::Call, InstType},
            FunPtr,
        },
        Program,
    },
    utils::traverse::{Node, POIterator},
};

pub struct CallGraph {
    main_fun: FunPtr,
}

impl CallGraph {
    pub fn new(program: &Program) -> Self {
        let main_fun = program
            .module
            .functions
            .iter()
            .find(|f| f.name == "main")
            .cloned()
            .unwrap();
        Self { main_fun }
    }

    pub fn po_iter(&self) -> impl Iterator<Item = CallGraphNode<'_>> {
        let node = CallGraphNode {
            fun: self.main_fun,
            context: self,
        };
        POIterator::from(node)
    }
}

#[derive(Clone)]
pub struct CallGraphNode<'a> {
    pub fun: FunPtr,
    context: &'a CallGraph,
}

impl<'a> PartialEq for CallGraphNode<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.fun == other.fun
    }
}

impl<'a> Eq for CallGraphNode<'a> {}

impl<'a> std::hash::Hash for CallGraphNode<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.fun.hash(state);
    }
}

impl<'a> Node for CallGraphNode<'a> {
    fn get_succ(&mut self) -> Vec<Self> {
        // Library function does not call other functions
        if self.fun.is_lib() {
            return vec![];
        }

        // Find all calls in the function
        let mut result = HashSet::new();
        for bb in self.fun.dfs_iter() {
            for inst in bb.iter() {
                if inst.get_type() == InstType::Call {
                    let call = downcast_ref::<Call>(inst.as_ref().as_ref());
                    result.insert(CallGraphNode {
                        fun: call.func,
                        context: self.context,
                    });
                }
            }
        }

        // Deduplicate set and return
        result.into_iter().collect()
    }
}
