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

pub struct EffectAnalysis {
    pub inst_effect: HashMap<InstPtr, Effect>,
    pub has_io_input: HashSet<FunPtr>,
    pub has_io_output: HashSet<FunPtr>,
    functions: Vec<FunPtr>,
    def_range: HashMap<FunPtr, HashSet<Operand>>,
    use_range: HashMap<FunPtr, HashSet<Operand>>,
}

#[allow(unused)]
impl EffectAnalysis {
    /// Run effect analysis on program.
    pub fn new(program: &Program) -> Self {
        let mut worklist: HashSet<FunPtr> = HashSet::new();
        let def_range = HashMap::new();
        let use_range = HashMap::new();
        let inst_effect = HashMap::new();
        let has_io_input = HashSet::new();
        let has_io_output = HashSet::new();
        let mut effect = Self {
            inst_effect,
            has_io_input,
            has_io_output,
            functions: program.module.functions.clone(),
            def_range,
            use_range,
        };

        // Set all library functions as has_io
        for func in program.module.functions.iter() {
            if func.is_lib() {
                if func.name.contains("get") {
                    effect.has_io_input.insert(*func);
                } else if func.name.contains("put") {
                    effect.has_io_output.insert(*func);
                } else if func.name.contains("memset") {
                    let def_range: HashSet<Operand> = [func.params[0].into()].into_iter().collect();
                    effect.def_range.insert(*func, def_range);
                }
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

    /// Get if function has memory load.
    pub fn has_load(&self, func: FunPtr) -> bool {
        self.use_range
            .get(&func)
            .map(HashSet::is_empty)
            .unwrap_or(true)
    }

    /// Get if function has memory store.
    pub fn has_store(&self, func: FunPtr) -> bool {
        self.def_range
            .get(&func)
            .map(HashSet::is_empty)
            .unwrap_or(true)
    }

    /// Get if instruction has IO.
    pub fn inst_has_io(&self, inst: InstPtr) -> bool {
        if inst.get_type() == InstType::Call {
            let call = downcast_ref::<Call>(inst.as_ref().as_ref());
            self.has_io(call.func)
        } else {
            false
        }
    }

    /// Get if function has IO.
    pub fn has_io(&self, func: FunPtr) -> bool {
        self.has_io_input.contains(&func) || self.has_io_output.contains(&func)
    }

    /// Get if function is pure function (no IO / load / store).
    pub fn is_pure(&self, func: FunPtr) -> bool {
        !self.has_io_input.contains(&func)
            && !self.has_io_output.contains(&func)
            && !self.has_load(func)
            && !self.has_store(func)
    }

    /// Get if function is constant function (no IO input / load).
    pub fn is_constant(&self, func: FunPtr) -> bool {
        !self.has_io_input.contains(&func) && !self.has_load(func)
    }

    /// Get if function is silent function (no IO output / store).
    pub fn is_silent(&self, func: FunPtr) -> bool {
        !self.has_io_output.contains(&func) && !self.has_store(func)
    }

    /// Dump effect analysis result to string.
    pub fn dump(&self) -> String {
        let mut res = String::new();
        for func in self.functions.iter() {
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
                    res += &format!("    def: {}\n", def_range.join(", "));
                    res += &format!("    use: {}\n\n", use_range.join(", "));
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
                        let has_io = self.has_io_input.contains(&call.func);

                        // Fill parameter in ranges
                        let fill = |op: Operand| {
                            if let Operand::Parameter(param) = op {
                                let mut params = call.func.params.iter();
                                if let Some(index) = params.position(|p| *p == param) {
                                    return inst.get_operand()[index].clone();
                                }
                            }
                            op
                        };
                        let def_range: HashSet<Operand> = def_range.into_iter().map(fill).collect();
                        let use_range: HashSet<Operand> = use_range.into_iter().map(fill).collect();

                        // Merge into function effect
                        changed |= self.add_batch_to_func(
                            func,
                            def_range.clone(),
                            use_range.clone(),
                            has_io,
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
                        changed |= self.set_has_io(func);

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
                        changed |= self.set_has_io(func);

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

    /// Merge effect of two functions, return changed or not.
    /// This attempts to change effect target from local variable to parameter.
    fn add_batch_to_func(
        &mut self,
        dst: FunPtr,
        def_range: HashSet<Operand>,
        use_range: HashSet<Operand>,
        has_io: bool,
    ) -> bool {
        let mut changed = false;
        for def_op in def_range {
            changed |= self.add_def_to_func(dst, def_op.clone());
        }
        for used_op in use_range {
            changed |= self.add_use_to_func(dst, used_op.clone());
        }
        if has_io {
            changed |= self.set_has_io(dst);
        }
        changed
    }

    fn make_global(op: Operand) -> Option<Operand> {
        if let Operand::Instruction(inst) = op {
            // Trace source of GEP
            if inst.get_type() == InstType::GetElementPtr {
                return Self::make_global(inst.get_operand()[0].clone());
            }

            // Otherwise it's alloca (local variable) and can't be made global
            return None;
        }

        // "Global", "Parameter" and "Constant" are all global
        Some(op)
    }

    /// Add memory def to function, return changed or not
    fn add_def_to_func(&mut self, func: FunPtr, target: Operand) -> bool {
        let Some(target) = Self::make_global(target) else {
            return false;
        };
        self.def_range.entry(func).or_default().insert(target)
    }

    /// Add memory use to function, return changed or not
    fn add_use_to_func(&mut self, func: FunPtr, target: Operand) -> bool {
        let Some(target) = Self::make_global(target) else {
            return false;
        };
        self.use_range.entry(func).or_default().insert(target)
    }

    /// Set has_ioness of function, return changed or not
    fn set_has_io(&mut self, func: FunPtr) -> bool {
        self.has_io_input.insert(func)
    }
}
