use std::collections::{HashMap, HashSet};

use crate::{
    backend::from_self::downcast_ref,
    middle::{
        ir::{
            instruction::{misc_inst::Call, InstType},
            FunPtr, InstPtr,
        },
        Program,
    },
    utils::traverse::{Node, POIterator},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CallEdge {
    pub inst: InstPtr,
    pub func: FunPtr,
}

#[allow(unused)]
pub struct CallGraph {
    main: FunPtr,
    calls: HashMap<FunPtr, HashSet<CallEdge>>,
    called_by: HashMap<FunPtr, HashSet<CallEdge>>,
}

impl CallGraph {
    pub fn new(program: &Program) -> Self {
        let mut calls = HashMap::new();
        let mut called_by = HashMap::new();
        let mut main = None;
        for func in program.module.functions.clone() {
            if func.name == "main" {
                main = Some(func);
            }
            if func.is_lib() {
                continue;
            }
            for bb in func.dfs_iter() {
                for inst in bb.iter() {
                    if inst.get_type() == InstType::Call {
                        let call = downcast_ref::<Call>(inst.as_ref().as_ref());
                        calls
                            .entry(func)
                            .or_insert(HashSet::new())
                            .insert(CallEdge {
                                inst,
                                func: call.func,
                            });
                        called_by
                            .entry(call.func)
                            .or_insert(HashSet::new())
                            .insert(CallEdge { inst, func });
                    }
                }
            }
        }
        CallGraph {
            main: main.unwrap(),
            calls,
            called_by,
        }
    }

    pub fn po_iter(&self) -> impl Iterator<Item = CallGraphNode<'_>> {
        let node = CallGraphNode {
            fun: self.main,
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
        self.context
            .calls
            .get(&self.fun)
            .unwrap_or(&HashSet::new())
            .iter()
            .map(|edge| CallGraphNode {
                fun: edge.func,
                context: self.context,
            })
            .collect()
    }
}

#[allow(unused)]
impl<'a> CallGraphNode<'a> {
    pub fn get_calls(&self) -> Vec<CallEdge> {
        self.context
            .calls
            .get(&self.fun)
            .unwrap_or(&HashSet::new())
            .iter()
            .cloned()
            .collect()
    }
    pub fn get_called_by(&self) -> Vec<CallEdge> {
        self.context
            .called_by
            .get(&self.fun)
            .unwrap_or(&HashSet::new())
            .iter()
            .cloned()
            .collect()
    }
}
