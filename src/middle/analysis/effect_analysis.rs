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

pub struct Effect {
    pub def_range: EffectRange,
    pub use_range: EffectRange,
}

pub struct EffectAnalysis {
    pub inst_effect: HashMap<InstPtr, Effect>,
    pub has_io_input: HashSet<FunPtr>,
    pub has_io_output: HashSet<FunPtr>,
    pub has_mem_input: HashSet<FunPtr>,
    pub has_mem_output: HashSet<FunPtr>,
    functions: Vec<FunPtr>,
}

impl EffectAnalysis {
    /// Run effect analysis on program.
    pub fn new(program: &Program) -> Self {
        let mut worklist: HashSet<FunPtr> = HashSet::new();
        let mut effect = Self {
            inst_effect: HashMap::new(),
            has_io_input: HashSet::new(),
            has_io_output: HashSet::new(),
            has_mem_input: HashSet::new(),
            has_mem_output: HashSet::new(),
            functions: program.module.functions.clone(),
        };

        // Set all library functions as has_io
        for func in program.module.functions.iter() {
            if func.is_lib() {
                if func.name.contains("get") {
                    effect.has_io_input.insert(*func);
                }
                if func.name.contains("put")
                    || func.name.contains("starttime")
                    || func.name.contains("stoptime")
                {
                    effect.has_io_output.insert(*func);
                }
                if func.name.contains("memset")
                    || func.name == "getarray"
                    || func.name == "getfarray"
                {
                    effect.has_mem_output.insert(*func);
                }
                if func.name == "putarray" || func.name == "putfarray" {
                    effect.has_mem_input.insert(*func);
                }
            } else {
                worklist.insert(*func);
            }
        }

        // Iterate all functions until unchanged
        let call_graph = CallGraph::new(program);
        loop {
            let mut changed = false;
            for func in program.module.functions.iter() {
                if func.is_lib() || !worklist.contains(func) {
                    continue;
                }
                worklist.remove(func);
                if effect.process_func(*func) {
                    changed = true;
                    worklist.extend(
                        call_graph
                            .get_called_by(*func)
                            .iter()
                            .map(|edge| edge.caller),
                    );
                }
            }
            if !changed {
                break effect;
            }
        }
    }

    /// Get if instruction has IO.
    pub fn has_io(&self, inst: InstPtr) -> bool {
        if inst.get_type() == InstType::Call {
            let call = downcast_ref::<Call>(inst.as_ref().as_ref());
            self.has_io_input.contains(&call.func) || self.has_io_output.contains(&call.func)
        } else {
            false
        }
    }

    /// Get if function has side effect.
    pub fn has_effect(&self, inst: InstPtr) -> bool {
        if inst.get_type() == InstType::Call {
            let call = downcast_ref::<Call>(inst.as_ref().as_ref());
            self.has_mem_input.contains(&call.func)
                || self.has_mem_output.contains(&call.func)
                || self.has_io_input.contains(&call.func)
                || self.has_io_output.contains(&call.func)
        } else {
            false
        }
    }

    /// Dump effect analysis result on instruction to string.
    pub fn dump_inst(&self) -> String {
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
                        if call.func.name.contains("memset") {
                            // Treat memset as a store
                            let ptr = get_base_pointer(inst.get_operand()[0].clone());
                            self.inst_effect.insert(
                                inst,
                                Effect {
                                    def_range: ptr.into(),
                                    use_range: EffectRange::new(),
                                },
                            );
                        } else if call.func.name == "putarray" || call.func.name == "putfarray" {
                            let ptr = get_base_pointer(inst.get_operand()[1].clone());
                            self.inst_effect.insert(
                                inst,
                                Effect {
                                    def_range: EffectRange::new(),
                                    use_range: ptr.into(),
                                },
                            );
                        } else if call.func.name == "getarray" || call.func.name == "getfarray" {
                            let ptr = get_base_pointer(inst.get_operand()[0].clone());
                            self.inst_effect.insert(
                                inst,
                                Effect {
                                    def_range: ptr.into(),
                                    use_range: EffectRange::new(),
                                },
                            );
                        } else if !call.func.is_lib() {
                            // Treat other non-library function as impure
                            self.inst_effect.insert(
                                inst,
                                Effect {
                                    def_range: EffectRange::All,
                                    use_range: EffectRange::new(),
                                },
                            );
                        }
                    }
                    InstType::Store => {
                        let target = inst.get_operand()[1].clone();
                        if check_effect(&target) {
                            changed |= self.has_mem_output.insert(func);
                        }

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
                        if check_effect(&target) {
                            changed |= self.has_mem_input.insert(func);
                        }

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
        if self.has_mem_input.contains(&src) {
            changed |= self.has_mem_input.insert(dst);
        }
        if self.has_mem_output.contains(&src) {
            changed |= self.has_mem_output.insert(dst);
        }
        changed
    }
}

/// Check if operand as store / load position causes outside effect.
fn check_effect(operand: &Operand) -> bool {
    match operand {
        Operand::Instruction(inst) => {
            if inst.get_type() == InstType::GetElementPtr {
                check_effect(inst.get_operand().first().unwrap())
            } else {
                false
            }
        }
        Operand::Global(_) => true,
        Operand::Parameter(_) => true,
        Operand::Constant(_) => false,
    }
}

/// Get base pointer of operand (because pointer in function argument can GEP a lot)
fn get_base_pointer(operand: Operand) -> Operand {
    match operand {
        Operand::Instruction(inst) => {
            if inst.get_type() == InstType::GetElementPtr {
                get_base_pointer(inst.get_operand().first().unwrap().clone())
            } else {
                operand
            }
        }
        _ => operand,
    }
}
