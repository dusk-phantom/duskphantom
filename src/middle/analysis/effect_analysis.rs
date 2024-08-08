use std::collections::{HashMap, HashSet};

use crate::{
    backend::from_self::downcast_ref,
    middle::{
        ir::{
            instruction::{misc_inst::Call, InstType},
            FunPtr, InstPtr, Operand,
        },
        Program,
    },
};

use super::call_graph::CallGraph;

#[allow(unused)]
pub struct Effect {
    pub def_range: HashSet<Operand>,
    pub use_range: HashSet<Operand>,
}

#[allow(unused)]
pub struct EffectAnalysis<'a> {
    pub inst_effect: HashMap<InstPtr, Effect>,
    pub impure: HashSet<FunPtr>,
    program: &'a Program,
    def_range: HashMap<FunPtr, HashSet<Operand>>,
    use_range: HashMap<FunPtr, HashSet<Operand>>,
}

#[allow(unused)]
impl<'a> EffectAnalysis<'a> {
    pub fn new(program: &'a Program) -> Self {
        let mut worklist: HashSet<FunPtr> = HashSet::new();
        let def_range = HashMap::new();
        let use_range = HashMap::new();
        let inst_effect = HashMap::new();
        let impure = HashSet::new();
        let mut effect = Self {
            inst_effect,
            impure,
            program,
            def_range,
            use_range,
        };

        // Set all library functions as impure
        for func in program.module.functions.iter() {
            if func.is_lib() {
                effect.impure.insert(*func);
            }
            worklist.insert(*func);
        }

        // Postorder iterate on call graph until unchanged
        let call_graph = CallGraph::new(program);
        loop {
            let mut changed = false;
            for node in call_graph.po_iter() {
                if node.fun.is_lib() || !worklist.contains(&node.fun) {
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

    pub fn dump(&self) -> String {
        let mut res = String::new();
        for func in self.program.module.functions.iter() {
            if func.is_lib() {
                continue;
            }
            for bb in func.dfs_iter() {
                for inst in bb.iter() {
                    let Some(effect) = self.inst_effect.get(&inst) else {
                        continue;
                    };
                    res += &format!("{}:\n", inst.gen_llvm_ir());
                    let mut def_range = effect
                        .def_range
                        .iter()
                        .map(|op| op.to_string())
                        .collect::<Vec<_>>();
                    let mut use_range = effect
                        .use_range
                        .iter()
                        .map(|op| op.to_string())
                        .collect::<Vec<_>>();
                    def_range.sort();
                    use_range.sort();
                    res += &format!("  def: {}\n", def_range.join(", "));
                    res += &format!("  use: {}\n\n", use_range.join(", "));
                }
            }
        }
        res
    }

    /// Process function, return changed or not
    fn process_func(&mut self, func: FunPtr) -> bool {
        let mut changed = false;
        for bb in func.dfs_iter() {
            for inst in bb.iter() {
                match inst.get_type() {
                    InstType::Call => {
                        let call = downcast_ref::<Call>(inst.as_ref().as_ref());
                        let def_range = self.def_range.get(&call.func).cloned().unwrap_or_default();
                        let use_range = self.use_range.get(&call.func).cloned().unwrap_or_default();
                        let impure = self.impure.contains(&call.func);

                        // Merge into function effect
                        changed |= self.add_batch_to_func(
                            func,
                            def_range.clone(),
                            use_range.clone(),
                            impure,
                        );

                        // Add call as an effective inst
                        self.inst_effect.insert(
                            inst,
                            Effect {
                                def_range,
                                use_range,
                            },
                        );
                    }
                    InstType::Store => {
                        let target = inst.get_operand()[1].clone();
                        changed |= self.add_def_to_func(func, target.clone());
                        changed |= self.set_impure(func);

                        // Add store as an effective inst
                        self.inst_effect.insert(
                            inst,
                            Effect {
                                def_range: [target].into_iter().collect(),
                                use_range: HashSet::new(),
                            },
                        );
                    }
                    InstType::Load => {
                        let target = inst.get_operand()[0].clone();
                        changed |= self.add_use_to_func(func, target.clone());
                        changed |= self.set_impure(func);

                        // Add load as an effective inst
                        self.inst_effect.insert(
                            inst,
                            Effect {
                                def_range: HashSet::new(),
                                use_range: [target].into_iter().collect(),
                            },
                        );
                    }
                    _ => {}
                }
            }
        }
        changed
    }

    /// Merge effect of two functions, return changed or not
    fn add_batch_to_func(
        &mut self,
        dst: FunPtr,
        def_range: HashSet<Operand>,
        use_range: HashSet<Operand>,
        impure: bool,
    ) -> bool {
        let mut changed = false;
        for def in def_range {
            changed |= self.add_def_to_func(dst, def.clone());
        }
        for use_ in use_range {
            changed |= self.add_use_to_func(dst, use_.clone());
        }
        if impure {
            changed |= self.set_impure(dst);
        }
        changed
    }

    /// Add memory def to function, return changed or not
    fn add_def_to_func(&mut self, func: FunPtr, target: Operand) -> bool {
        self.def_range.entry(func).or_default().insert(target)
    }

    /// Add memory use to function, return changed or not
    fn add_use_to_func(&mut self, func: FunPtr, target: Operand) -> bool {
        self.use_range.entry(func).or_default().insert(target)
    }

    /// Set impureness of function, return changed or not
    fn set_impure(&mut self, func: FunPtr) -> bool {
        self.impure.insert(func)
    }
}
