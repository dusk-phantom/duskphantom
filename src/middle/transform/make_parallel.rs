use std::collections::{HashMap, HashSet};

use anyhow::Result;

use crate::{
    backend::from_self::downcast_ref,
    middle::{
        analysis::{
            effect_analysis::{Effect, EffectAnalysis},
            loop_tools::{LoopForest, LoopPtr},
        },
        ir::{
            instruction::{
                misc_inst::{ICmp, ICmpOp, Phi},
                InstType,
            },
            BBPtr, Constant, InstPtr, Operand, ValueType,
        },
        Program,
    },
};

use super::Transform;

pub fn optimize_program<'a>(
    program: &'a mut Program,
    loop_forest: &'a mut LoopForest,
) -> Result<bool> {
    let effect_analysis = EffectAnalysis::new(program);
    MakeParallel::<5>::new(program, loop_forest, &effect_analysis).run_and_log()
}

pub struct MakeParallel<'a, const N_THREAD: usize> {
    program: &'a mut Program,
    loop_forest: &'a mut LoopForest,
    effect_analysis: &'a EffectAnalysis,
    has_return: HashSet<LoopPtr>,
    loop_effect: HashMap<LoopPtr, Effect>,
}

impl<'a, const N_THREAD: usize> Transform for MakeParallel<'a, N_THREAD> {
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

impl<'a, const N_THREAD: usize> MakeParallel<'a, N_THREAD> {
    pub fn new(
        program: &'a mut Program,
        loop_forest: &'a mut LoopForest,
        effect_analysis: &'a EffectAnalysis,
    ) -> Self {
        Self {
            program,
            loop_forest,
            effect_analysis,
            has_return: HashSet::new(),
            loop_effect: HashMap::new(),
        }
    }

    /// Preprocess each loop. This:
    /// - Check if any block has `exit` as succ, store to `has_return`
    /// - Get merged effect of each block if there's no collision, store to `loop_effect`
    fn preprocess(&mut self, lo: LoopPtr) -> Result<()> {
        let mut has_return = false;
        let mut loop_effect = Some(Effect::new());

        // If any of sub loops has return, then this loop has return;
        // Effect of sub loops are owned by this loop;
        // We need to compute all sub loops here, because it will be cached
        for sub_loop in lo.sub_loops.iter() {
            self.preprocess(*sub_loop)?;
            if self.has_return.contains(sub_loop) {
                has_return = true;
            }
            if let Some(effect) = &mut loop_effect {
                if let Some(sub_effect) = self.loop_effect.get(sub_loop) {
                    if !merge_effect(effect, sub_effect)? {
                        loop_effect = None;
                    }
                }
            }
        }

        // Check if any block has `exit` as succ, store to `has_return`
        for bb in &lo.blocks {
            if bb.get_succ_bb().iter().any(|&bb| bb.name == "exit") {
                has_return = true;
                break;
            }
        }

        // Get merged effect of each block if there's no collision, store to `loop_effect`
        for bb in &lo.blocks {
            let block_effect = self.get_block_effect(*bb)?;
            if let Some(block_effect) = block_effect {
                if let Some(effect) = &mut loop_effect {
                    if !merge_effect(effect, &block_effect)? {
                        loop_effect = None;
                        break;
                    }
                }
            }
        }

        // Store result
        if has_return {
            self.has_return.insert(lo);
        }
        if let Some(effect) = loop_effect {
            self.loop_effect.insert(lo, effect);
        }
        Ok(())
    }

    fn get_block_effect(&mut self, bb: BBPtr) -> Result<Option<Effect>> {
        let mut effect = Effect::new();
        for inst in bb.iter() {
            let inst_effect = &self.effect_analysis.inst_effect[&inst];
            if !merge_effect(&mut effect, inst_effect)? {
                return Ok(None);
            }
        }
        Ok(Some(effect))
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

    fn make_parallel(&mut self, mut candidate: ParallelCandidate) -> Result<bool> {
        // Get current thread count
        // TODO create library function
        // TODO fill argument correctly
        let func = self
            .program
            .module
            .functions
            .iter()
            .find(|f| f.name == "thrd_create")
            .unwrap();
        let inst_create = self
            .program
            .mem_pool
            .get_call(*func, vec![Constant::Int(0).into()]);
        candidate.indvar.init_bb.push_back(inst_create);

        // Create parallized exit and indvar
        //
        // i = init_val
        // d = next_delta
        // e = exit_value
        // n = current_thread
        // N = N_THREAD
        //
        // Before: i, i + d, i + 2d, ..., i + d(X = (e - i) ceildiv d)
        // After: [ LB = i + (e-i)n/N, UB = i + ((e-i)n + e-i)/N )
        // TODO sign preserve
        let i = candidate.indvar.init_val;
        let d = candidate.indvar.next_delta;
        let e = candidate.indvar.exit_val;
        let inst_mul = self
            .program
            .mem_pool
            .get_mul(inst_create.into(), Constant::Int(e - i).into());
        candidate.indvar.init_bb.push_back(inst_mul);
        let inst_div = self
            .program
            .mem_pool
            .get_sdiv(inst_mul.into(), Constant::Int(N_THREAD as i32).into());
        candidate.indvar.init_bb.push_back(inst_div);
        let inst_lb = self
            .program
            .mem_pool
            .get_add(inst_div.into(), Constant::Int(i).into());
        candidate.indvar.init_bb.push_back(inst_lb);
        let inst_add = self
            .program
            .mem_pool
            .get_add(inst_mul.into(), Constant::Int(e - i).into());
        candidate.indvar.init_bb.push_back(inst_add);
        let inst_div = self
            .program
            .mem_pool
            .get_sdiv(inst_add.into(), Constant::Int(N_THREAD as i32).into());
        candidate.indvar.init_bb.push_back(inst_div);
        let inst_ub = self
            .program
            .mem_pool
            .get_add(inst_div.into(), Constant::Int(i).into());
        candidate.indvar.init_bb.push_back(inst_ub);
        // TODO replace ind var
        let inst_cond = self.program.mem_pool.get_icmp(
            ICmpOp::Slt,
            ValueType::Int,
            candidate.indvar.indvar.into(),
            Constant::Int(e).into(),
        );
        candidate.indvar.exit.set_operand(0, inst_cond.into());

        // TODO Join threads
        Ok(true)
    }
}

/// Merge effect if parallelizing them doesn't cause collision.
/// Returns changed or not.
fn merge_effect(a: &mut Effect, b: &Effect) -> Result<bool> {
    if a.def_range.can_conflict(&b.def_range)
        || a.use_range.can_conflict(&b.def_range)
        || a.def_range.can_conflict(&b.use_range)
    {
        return Ok(false);
    }
    a.def_range.merge(&b.def_range);
    a.use_range.merge(&b.use_range);
    Ok(true)
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
/// exit = br (indvar < 6), loop, exit
/// indvar = phi [2, pre_header], [indvar + 3, loop]
/// ```
///
/// indvar = phi [2, pre_header], [indvar + 3, loop]
/// exit = br (indvar < 6), loop, exit
/// init_val = 2
/// init_bb = pre_header
/// next_delta = 3
/// next_bb = loop
/// exit_val = 6
struct IndVar {
    indvar: InstPtr,
    exit: InstPtr,
    init_val: i32,
    init_bb: BBPtr,
    next_delta: i32,
    next_bb: BBPtr,
    exit_val: i32,
    exit_bb: BBPtr,
}

impl IndVar {
    fn new(
        indvar: InstPtr,
        exit: InstPtr,
        init_val: i32,
        init_bb: BBPtr,
        next_delta: i32,
        next_bb: BBPtr,
        exit_val: i32,
        exit_bb: BBPtr,
    ) -> Self {
        Self {
            indvar,
            exit,
            init_val,
            init_bb,
            next_delta,
            next_bb,
            exit_val,
            exit_bb,
        }
    }

    /// Get induction variable from exit instruction.
    /// Exit instruction should shape like:
    /// `exit = br (indvar < N), loop, exit`
    /// TODO-WA: check if indvar delta direction is compatible with exit criteria
    fn from_exit(exit: InstPtr, lo: LoopPtr) -> Option<Self> {
        if exit.get_type() != InstType::Br {
            return None;
        }
        let parent_bb = exit.get_parent_bb()?;
        let exit_bb = parent_bb
            .get_succ_bb()
            .iter()
            .find(|bb| !lo.is_in_loop(bb))?;
        let Operand::Instruction(cond) = exit.get_operand().first()? else {
            return None;
        };
        let InstType::ICmp = cond.get_type() else {
            return None;
        };
        let icmp = downcast_ref::<ICmp>(cond.as_ref().as_ref());
        if icmp.op != ICmpOp::Slt || icmp.op != ICmpOp::Sgt {
            return None;
        }
        let Operand::Instruction(indvar) = icmp.get_lhs() else {
            return None;
        };
        let Operand::Constant(Constant::Int(exit_val)) = icmp.get_rhs() else {
            return None;
        };
        if indvar.get_type() != InstType::Phi {
            return None;
        }
        let phi = downcast_ref::<Phi>(indvar.as_ref().as_ref());
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
            *indvar, exit, init_val, init_bb, next_delta, next_bb, *exit_val, *exit_bb,
        ))
    }
}
