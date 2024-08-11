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
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CallEdge {
    pub inst: InstPtr,
    pub caller: FunPtr,
    pub callee: FunPtr,
}

#[allow(unused)]
pub struct CallGraph {
    main: Option<FunPtr>,
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

            // Caller should not be library function
            if func.is_lib() {
                continue;
            }

            // Iterate all instructions
            for bb in func.dfs_iter() {
                for inst in bb.iter() {
                    if inst.get_type() == InstType::Call {
                        let call = downcast_ref::<Call>(inst.as_ref().as_ref());

                        // Callee should not be library function
                        if call.func.is_lib() {
                            continue;
                        }

                        // Construct and add call edge
                        let call_edge = CallEdge {
                            inst,
                            caller: func,
                            callee: call.func,
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
            main,
            calls,
            called_by,
        }
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
