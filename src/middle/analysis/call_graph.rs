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
    pub caller: FunPtr,
    pub callee: FunPtr,
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
                        let call_edge = CallEdge {
                            inst,
                            caller: call.func,
                            callee: func,
                        };
                        calls
                            .entry(func)
                            .or_insert(HashSet::new())
                            .insert(call_edge);
                        called_by
                            .entry(call.func)
                            .or_insert(HashSet::new())
                            .insert(call_edge);
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
            func: self.main,
            context: self,
        };
        POIterator::from(node)
    }

    pub fn get_calls(&self, func: FunPtr) -> HashSet<CallEdge> {
        self.calls.get(&func).cloned().unwrap_or_default()
    }

    pub fn get_called_by(&self, func: FunPtr) -> HashSet<CallEdge> {
        self.called_by.get(&func).cloned().unwrap_or_default()
    }

    pub fn remove(&mut self, func: FunPtr) {
        if let Some(calls) = self.calls.remove(&func) {
            for call in calls {
                self.called_by.get_mut(&call.caller).unwrap().remove(&call);
            }
        }
        if let Some(called_by) = self.called_by.remove(&func) {
            for call in called_by {
                self.calls.get_mut(&call.caller).unwrap().remove(&call);
            }
        }
    }
}

#[derive(Clone)]
pub struct CallGraphNode<'a> {
    pub func: FunPtr,
    context: &'a CallGraph,
}

impl<'a> PartialEq for CallGraphNode<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.func == other.func
    }
}

impl<'a> Eq for CallGraphNode<'a> {}

impl<'a> std::hash::Hash for CallGraphNode<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.func.hash(state);
    }
}

impl<'a> Node for CallGraphNode<'a> {
    fn get_succ(&mut self) -> Vec<Self> {
        self.context
            .calls
            .get(&self.func)
            .unwrap_or(&HashSet::new())
            .iter()
            .map(|edge| CallGraphNode {
                func: edge.callee,
                context: self.context,
            })
            .collect()
    }
}
