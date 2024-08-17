use std::collections::{HashMap, HashSet};

use anyhow::Result;

use crate::{
    backend::from_self::downcast_ref,
    middle::{
        analysis::{
            effect_analysis::Effect,
            loop_tools::{LoopForest, LoopPtr},
        },
        ir::{
            instruction::{
                misc_inst::{ICmp, ICmpOp, Phi},
                InstType,
            },
            BBPtr, Constant, InstPtr, Operand,
        },
        Program,
    },
};

use super::Transform;

pub fn optimize_program<'a>(
    program: &'a mut Program,
    loop_forest: &'a mut LoopForest,
) -> Result<bool> {
    MakeParallel::new(program, loop_forest).run_and_log()
}

pub struct MakeParallel<'a> {
    program: &'a mut Program,
    loop_forest: &'a mut LoopForest,
    has_return: HashSet<LoopPtr>,
    loop_effect: HashMap<LoopPtr, Effect>,
}

impl<'a> Transform for MakeParallel<'a> {
    fn get_program_mut(&mut self) -> &mut Program {
        self.program
    }

    fn name() -> String {
        "make_parallel".to_string()
    }

    fn run(&mut self) -> Result<bool> {
        let mut changed = false;
        for lo in self.loop_forest.forest.clone() {
            let mut candidate = Vec::new();
            self.make_candidate(&mut candidate, lo)?;
            for c in candidate {
                changed |= self.make_parallel(c)?;
            }
        }
        Ok(changed)
    }
}

impl<'a> MakeParallel<'a> {
    pub fn new(program: &'a mut Program, loop_forest: &'a mut LoopForest) -> Self {
        Self {
            program,
            loop_forest,
            has_return: HashSet::new(),
            loop_effect: HashMap::new(),
        }
    }

    /// Preprocess each loop. This:
    /// - Check if any block has `exit` as succ, store to `has_return`
    /// - Get merged effect of each block if there's no collision, store to `loop_effect`
    fn preprocess(&mut self, lo: LoopPtr) -> Result<()> {
        let mut has_return = false;

        // If any of sub loops has return, then this loop has return
        // However, we need to compute all sub loops here, because it will be cached
        for sub_loop in lo.sub_loops.iter() {
            self.preprocess(*sub_loop)?;
            if self.has_return.contains(sub_loop) {
                has_return = true;
            }
        }

        // Check if any block has `exit` as succ, store to `has_return`
        for bb in &lo.blocks {
            if bb.get_succ_bb().iter().any(|&bb| bb.name == "exit") {
                has_return = true;
            }
        }

        // TODO-funct: get merged effect of each block if there's no collision, store to `loop_effect`

        // Store result
        if has_return {
            self.has_return.insert(lo);
        }
        Ok(())
    }

    fn make_candidate(&mut self, result: &mut Vec<ParallelCandidate>, lo: LoopPtr) -> Result<()> {
        // If loop has return, then it can't be parallelized, check sub loops instead
        if self.has_return.contains(&lo) {
            for lo in lo.sub_loops.iter() {
                self.make_candidate(result, *lo)?;
            }
            return Ok(());
        }

        // If effect range collides, then it can't be parallelized, check sub loops instead
        if !self.loop_effect.contains_key(&lo) {
            for lo in lo.sub_loops.iter() {
                self.make_candidate(result, *lo)?;
            }
            return Ok(());
        }

        // Get all exit edges
        // TODO-TLE: ignore all bb with one succ
        let mut exit = Vec::new();
        for bb in &lo.blocks {
            if bb.get_succ_bb().iter().any(|bb| !lo.is_in_loop(bb)) {
                exit.push(bb.get_last_inst());
            }
        }

        // If there are multiple exit edges, then it can't be parallelized
        if exit.len() != 1 {
            for lo in lo.sub_loops.iter() {
                self.make_candidate(result, *lo)?;
            }
            return Ok(());
        }

        // Get induction var from exit. If failed, check sub loops instead
        let exit = exit[0];
        let Some(indvar) = IndVar::from_exit(exit, lo) else {
            for lo in lo.sub_loops.iter() {
                self.make_candidate(result, *lo)?;
            }
            return Ok(());
        };

        // Insert candidate to results
        result.push(ParallelCandidate::new(lo, indvar));
        Ok(())
    }

    fn make_parallel(&mut self, candidate: ParallelCandidate) -> Result<bool> {
        todo!("create parallized exit and indvar, join threads");
    }
}

/// A candidate for parallelization.
/// For example:
///
/// ```c
/// int i = 2;
/// while (i < 6) {
///     // body
///     i += 2;
/// }
/// ```
///
/// exit = br (indvar < 6), loop, exit
/// indvar = phi [2, pre_header], [indvar + 2, loop]
struct ParallelCandidate {
    lo: LoopPtr,
    indvar: IndVar,
}

impl ParallelCandidate {
    fn new(lo: LoopPtr, indvar: IndVar) -> Self {
        Self { lo, indvar }
    }
}

/// An inductive variable.
/// For example:
///
/// ```llvm
/// indvar = phi [2, pre_header], [indvar + 3, loop]
/// ```
///
/// inst = phi [2, pre_header], [indvar + 3, loop]
/// init_val = 2
/// init_bb = pre_header
/// next_delta = 3
/// next_bb = loop
struct IndVar {
    inst: InstPtr,
    init_val: i32,
    init_bb: BBPtr,
    next_delta: i32,
    next_bb: BBPtr,
    exit_val: i32,
    exit_inst: InstPtr,
}

impl IndVar {
    fn new(
        inst: InstPtr,
        init_val: i32,
        init_bb: BBPtr,
        next_delta: i32,
        next_bb: BBPtr,
        exit_val: i32,
        exit_inst: InstPtr,
    ) -> Self {
        Self {
            inst,
            init_val,
            init_bb,
            next_delta,
            next_bb,
            exit_val,
            exit_inst,
        }
    }

    /// Get induction variable from exit instruction.
    /// Exit instruction should shape like:
    /// `exit = br (indvar < N), loop, exit`
    /// TODO-WA: check if indvar delta direction is compatible with exit criteria
    fn from_exit(exit_inst: InstPtr, lo: LoopPtr) -> Option<Self> {
        let Operand::Instruction(cond) = exit_inst.get_operand().first()? else {
            return None;
        };
        let InstType::ICmp = cond.get_type() else {
            return None;
        };
        let icmp = downcast_ref::<ICmp>(cond.as_ref().as_ref());
        if icmp.op != ICmpOp::Slt || icmp.op != ICmpOp::Sgt {
            return None;
        }
        let Operand::Instruction(inst) = icmp.get_lhs() else {
            return None;
        };
        let Operand::Constant(Constant::Int(exit_val)) = icmp.get_rhs() else {
            return None;
        };
        Self::from_phi(*inst, lo, *exit_val, exit_inst)
    }

    /// Get inductive variable from a phi instruction.
    /// Phi instruction should shape like:
    /// `indvar = phi [2, pre_header], [indvar + 3, loop]`
    fn from_phi(inst: InstPtr, lo: LoopPtr, exit_val: i32, exit_inst: InstPtr) -> Option<Self> {
        if inst.get_type() != InstType::Phi {
            return None;
        }
        let phi = downcast_ref::<Phi>(inst.as_ref().as_ref());
        let inc = phi.get_incoming_values();
        if inc.len() != 2 {
            return None;
        }
        let Operand::Constant(Constant::Int(init_val)) = inc[0].0 else {
            return None;
        };
        let init_bb = lo.pre_header?;
        if init_bb != inc[0].1 {
            return None;
        }
        let Operand::Instruction(next_val) = inc[1].0 else {
            return None;
        };
        let next_delta = match next_val.get_type() {
            InstType::Add => {
                let Operand::Constant(Constant::Int(delta)) = next_val.get_operand()[1] else {
                    return None;
                };
                Some(delta)
            }
            InstType::Sub => {
                let Operand::Constant(Constant::Int(delta)) = next_val.get_operand()[1] else {
                    return None;
                };
                Some(-delta)
            }
            _ => None,
        }?;
        let next_bb = inc[1].1;
        if !lo.is_in_loop(&next_bb) {
            return None;
        }
        Some(Self::new(
            inst, init_val, init_bb, next_delta, next_bb, exit_val, exit_inst,
        ))
    }
}
