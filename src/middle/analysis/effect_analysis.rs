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

use super::{alias_analysis::EffectRange, call_graph::CallGraph};

#[allow(unused)]
pub struct Effect {
    pub def_range: EffectRange,
    pub use_range: EffectRange,
}

pub struct EffectAnalysis {
    pub inst_effect: HashMap<InstPtr, Effect>,
    has_io_input: HashSet<FunPtr>,
    has_io_output: HashSet<FunPtr>,
    has_load: HashSet<FunPtr>,
    has_store: HashSet<FunPtr>,
    functions: Vec<FunPtr>,
}

#[allow(unused)]
impl EffectAnalysis {
    /// Run effect analysis on program.
    pub fn new(program: &Program) -> Self {
        let mut worklist: HashSet<FunPtr> = HashSet::new();
        let inst_effect = HashMap::new();
        let has_io_input = HashSet::new();
        let has_io_output = HashSet::new();
        let has_load = HashSet::new();
        let has_store = HashSet::new();
        let mut effect = Self {
            inst_effect,
            has_io_input,
            has_io_output,
            has_load,
            has_store,
            functions: program.module.functions.clone(),
        };

        // Set all library functions as has_io
        for func in program.module.functions.iter() {
            if func.is_lib() {
                if func.name.contains("get") {
                    effect.has_io_input.insert(*func);
                } else if func.name.contains("put") {
                    effect.has_io_output.insert(*func);
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
            && !self.has_load.contains(&func)
            && !self.has_store.contains(&func)
    }

    /// Get if function is constant function (no IO input / load).
    pub fn is_constant(&self, func: FunPtr) -> bool {
        !self.has_io_input.contains(&func) && !self.has_load.contains(&func)
    }

    /// Get if function is silent function (no IO output / store).
    pub fn is_silent(&self, func: FunPtr) -> bool {
        !self.has_io_output.contains(&func) && !self.has_store.contains(&func)
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
                    res += &format!("    def: {}\n", effect.def_range.dump());
                    res += &format!("    use: {}\n\n", effect.use_range.dump());
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
                        // Merge function effect
                        let call = downcast_ref::<Call>(inst.as_ref().as_ref());
                        changed |= self.add_batch_to_func(func, call.func);

                        // Add instruction effect
                        if call.func.is_memset() {
                            // Treat memset as a store
                            self.inst_effect.insert(
                                inst,
                                Effect {
                                    def_range: inst.get_operand()[0].clone().into(),
                                    use_range: EffectRange::new(),
                                },
                            );
                        } else if !call.func.is_lib() {
                            // Treat other non-library function as impure
                            self.inst_effect.insert(
                                inst,
                                Effect {
                                    def_range: EffectRange::All,
                                    use_range: EffectRange::All,
                                },
                            );
                        }
                    }
                    InstType::Store => {
                        let target = inst.get_operand()[1].clone();
                        changed |= self.has_store.insert(func);

                        // Add store as an effective inst
                        self.inst_effect.insert(
                            inst,
                            Effect {
                                def_range: target.into(),
                                use_range: EffectRange::new(),
                            },
                        );
                    }
                    InstType::Load => {
                        let target = inst.get_operand()[0].clone();
                        changed |= self.has_load.insert(func);

                        // Add load as an effective inst
                        self.inst_effect.insert(
                            inst,
                            Effect {
                                def_range: EffectRange::new(),
                                use_range: target.into(),
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
    fn add_batch_to_func(&mut self, dst: FunPtr, src: FunPtr) -> bool {
        let mut changed = false;
        if self.has_io_input.contains(&src) {
            changed |= self.has_io_input.insert(dst);
        }
        if self.has_io_output.contains(&src) {
            changed |= self.has_io_output.insert(dst);
        }
        if self.has_load.contains(&src) {
            changed |= self.has_load.insert(dst);
        }
        if self.has_store.contains(&src) {
            changed |= self.has_store.insert(dst);
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

    /// Set has_io_input of function, return changed or not
    fn set_has_io_input(&mut self, func: FunPtr) -> bool {
        self.has_io_input.insert(func)
    }
}
