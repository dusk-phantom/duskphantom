use std::collections::{HashMap, HashSet};

use crate::{
    backend::from_self::downcast_ref,
    middle::{
        ir::{
            instruction::{misc_inst::Call, InstType},
            FunPtr, Operand,
        },
        Program,
    },
};

use super::call_graph::CallGraph;

#[allow(unused)]
pub struct Effect {
    mem_def: HashMap<FunPtr, HashSet<Operand>>,
    mem_use: HashMap<FunPtr, HashSet<Operand>>,
    impure: HashSet<FunPtr>,
}

#[allow(unused)]
impl Effect {
    pub fn new(program: &Program) -> Self {
        let mut worklist: HashSet<FunPtr> = HashSet::new();
        let mem_def = HashMap::new();
        let mem_use = HashMap::new();
        let impure = HashSet::new();
        let mut effect = Self {
            mem_def,
            mem_use,
            impure,
        };

        // Set all library functions as impure
        for func in program.module.functions.iter() {
            if func.is_lib() {
                effect.impure.insert(*func);
            }
        }

        // Postorder iterate on call graph until unchanged
        let call_graph = CallGraph::new(program);
        loop {
            let mut changed = false;
            for node in call_graph.po_iter() {
                if !worklist.contains(&node.fun) {
                    continue;
                }
                worklist.remove(&node.fun);
                if effect.process_func(node.fun) {
                    changed = true;
                    worklist.extend(node.get_called_by().iter().map(|edge| edge.func));
                }
            }
            if !changed {
                break effect;
            }
        }
    }

    /// Process function, return changed or not
    fn process_func(&mut self, func: FunPtr) -> bool {
        let mut changed = false;
        for bb in func.dfs_iter() {
            for inst in bb.iter() {
                match inst.get_type() {
                    InstType::Call => {
                        let call = downcast_ref::<Call>(inst.as_ref().as_ref());
                        changed |= self.merge_func(func, call.func);
                    }
                    InstType::Store => {
                        let target = inst.get_operand()[1].clone();
                        changed |= self.add_def_to_func(func, target);
                        changed |= self.set_impure(func);
                    }
                    InstType::Load => {
                        let target = inst.get_operand()[0].clone();
                        changed |= self.add_use_to_func(func, target);
                        changed |= self.set_impure(func);
                    }
                    _ => {}
                }
            }
        }
        changed
    }

    /// Merge effect of two functions, return changed or not
    fn merge_func(&mut self, dst: FunPtr, src: FunPtr) -> bool {
        let mut changed = false;
        if let Some(defs) = self.mem_def.get(&src) {
            for def in defs.clone() {
                changed |= self.add_def_to_func(dst, def.clone());
            }
        }
        if let Some(uses) = self.mem_use.get(&src) {
            for use_ in uses.clone() {
                changed |= self.add_use_to_func(dst, use_.clone());
            }
        }
        if self.impure.contains(&src) {
            changed |= self.set_impure(dst);
        }
        changed
    }

    /// Add memory def to function, return changed or not
    fn add_def_to_func(&mut self, func: FunPtr, target: Operand) -> bool {
        self.mem_def.entry(func).or_default().insert(target)
    }

    /// Add memory use to function, return changed or not
    fn add_use_to_func(&mut self, func: FunPtr, target: Operand) -> bool {
        self.mem_use.entry(func).or_default().insert(target)
    }

    /// Set impureness of function, return changed or not
    fn set_impure(&mut self, func: FunPtr) -> bool {
        self.impure.insert(func)
    }
}
