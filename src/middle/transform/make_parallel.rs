use std::collections::{HashMap, HashSet};

use anyhow::Result;

use crate::{
    backend::from_self::downcast_ref,
    middle::{
        analysis::{
            dominator_tree::DominatorTree,
            effect_analysis::{Effect, EffectAnalysis},
            loop_tools::{self, LoopForest, LoopPtr},
        },
        ir::{
            instruction::{
                downcast_mut,
                misc_inst::{ICmp, ICmpOp, Phi},
                InstType,
            },
            BBPtr, Constant, InstPtr, Operand, ValueType,
        },
        Program,
    },
};

use super::{loop_simplify, Transform};

pub fn optimize_program<const N_THREAD: i32>(program: &mut Program) -> Result<bool> {
    let mut changed = false;
    let effect_analysis = EffectAnalysis::new(program);
    for func in program.module.functions.clone() {
        let Some(mut forest) = loop_tools::LoopForest::make_forest(func) else {
            continue;
        };
        loop_simplify::LoopSimplifier::new(&mut program.mem_pool).run(&mut forest)?;
        let mut dom_tree = DominatorTree::new(func);
        changed |=
            MakeParallel::<N_THREAD>::new(program, &mut forest, &mut dom_tree, &effect_analysis)
                .run()?;
    }
    Ok(changed)
}

pub struct MakeParallel<'a, const N_THREAD: i32> {
    program: &'a mut Program,
    loop_forest: &'a mut LoopForest,
    dom_tree: &'a mut DominatorTree,
    effect_analysis: &'a EffectAnalysis,
    has_return: HashSet<LoopPtr>,
    stack_ref: HashMap<LoopPtr, HashSet<InstPtr>>,
}

impl<'a, const N_THREAD: i32> Transform for MakeParallel<'a, N_THREAD> {
    fn get_program_mut(&mut self) -> &mut Program {
        self.program
    }

    fn name() -> String {
        "make_parallel".to_string()
    }

    fn run(&mut self) -> Result<bool> {
        let mut changed = false;
        for lo in self.loop_forest.forest.clone() {
            self.check_has_return(lo)?;
            self.check_stack_reference(lo)?;
        }
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

impl<'a, const N_THREAD: i32> MakeParallel<'a, N_THREAD> {
    pub fn new(
        program: &'a mut Program,
        loop_forest: &'a mut LoopForest,
        dom_tree: &'a mut DominatorTree,
        effect_analysis: &'a EffectAnalysis,
    ) -> Self {
        Self {
            program,
            loop_forest,
            dom_tree,
            effect_analysis,
            has_return: HashSet::new(),
            stack_ref: HashMap::new(),
        }
    }

    /// Check if any block has `exit` as succ, store to `has_return`
    fn check_has_return(&mut self, lo: LoopPtr) -> Result<()> {
        let mut has_return = false;

        // If any of sub loops has return, then this loop has return;
        // Effect of sub loops are owned by this loop;
        // We need to compute all sub loops here, because it will be cached
        for sub_loop in lo.sub_loops.iter() {
            self.check_has_return(*sub_loop)?;
            if self.has_return.contains(sub_loop) {
                has_return = true;
            }
        }

        // Check if any block has `exit` as succ, store to `has_return`
        for bb in &lo.blocks {
            if bb.get_succ_bb().iter().any(|&bb| bb.name == "exit") {
                has_return = true;
                break;
            }
        }

        // Store result
        if has_return {
            self.has_return.insert(lo);
        }
        Ok(())
    }

    fn check_stack_reference(&mut self, lo: LoopPtr) -> Result<()> {
        for bb in &lo.blocks {
            for inst in bb.iter() {
                if let Some(inst) = get_base_alloc(inst) {
                    self.stack_ref.entry(lo).or_default().insert(inst);
                }
            }
        }
        for sub_loop in lo.sub_loops.iter() {
            self.check_stack_reference(*sub_loop)?;
            let Some(sub_ref) = self.stack_ref.get(sub_loop).cloned() else {
                continue;
            };
            self.stack_ref.entry(lo).or_default().extend(sub_ref);
        }
        Ok(())
    }

    fn get_loop_effect(&mut self, lo: LoopPtr, indvar: &Operand) -> Result<Option<Effect>> {
        let mut effect = Effect::new();
        for bb in &lo.blocks {
            let Some(bb_effect) = self.get_block_effect(*bb, indvar)? else {
                return Ok(None);
            };
            if !merge_effect(&mut effect, &bb_effect, indvar)? {
                return Ok(None);
            }
        }

        // Additionally collect effect in sub loops
        for sub_loop in lo.sub_loops.iter() {
            let Some(sub_effect) = self.get_loop_effect(*sub_loop, indvar)? else {
                return Ok(None);
            };
            if !merge_effect(&mut effect, &sub_effect, indvar)? {
                return Ok(None);
            }
        }
        Ok(Some(effect))
    }

    fn get_block_effect(&mut self, bb: BBPtr, indvar: &Operand) -> Result<Option<Effect>> {
        let mut effect = Effect::new();
        for inst in bb.iter() {
            if let Some(inst_effect) = &self.effect_analysis.inst_effect.get(&inst) {
                if !merge_effect(&mut effect, inst_effect, indvar)? {
                    return Ok(None);
                }
            }
        }
        Ok(Some(effect))
    }

    fn make_candidate(&mut self, result: &mut Vec<Candidate>, lo: LoopPtr) -> Result<()> {
        // If loop has return, then it can't be parallelized, check sub loops instead
        let pre_header = lo.pre_header.unwrap();
        if self.has_return.contains(&lo) {
            println!("[INFO] loop {} has return", pre_header.name);
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
            println!("[INFO] loop {} has multiple exit edges", pre_header.name);
            for lo in lo.sub_loops.iter() {
                self.make_candidate(result, *lo)?;
            }
            return Ok(());
        }

        // Get induction var from exit. If failed, check sub loops instead
        let exit = exit[0];
        let Some(candidate) = Candidate::from_exit(exit, lo, self.dom_tree) else {
            println!("[INFO] loop {} does not have indvar", pre_header.name);
            for lo in lo.sub_loops.iter() {
                self.make_candidate(result, *lo)?;
            }
            return Ok(());
        };

        // If effect range collides, then it can't be parallelized, check sub loops instead
        if self
            .get_loop_effect(lo, &candidate.indvar.into())?
            .is_none()
        {
            println!("[INFO] loop {} has conflict effect", pre_header.name);
            for lo in lo.sub_loops.iter() {
                self.make_candidate(result, *lo)?;
            }
            return Ok(());
        }

        // Insert candidate to results
        println!(
            "[INFO] loop {} is made candidate {}!",
            pre_header.name,
            candidate.dump()
        );
        result.push(candidate);
        Ok(())
    }

    fn make_parallel(&mut self, mut candidate: Candidate) -> Result<bool> {
        // Copy global array address to local stack
        let mut map = HashMap::new();
        if let Some(stack_ref) = self.stack_ref.get(&candidate.lo) {
            for inst in stack_ref.iter() {
                let gep_zero = self.program.mem_pool.get_getelementptr(
                    inst.get_value_type().get_sub_type().cloned().unwrap(),
                    (*inst).into(),
                    vec![Constant::Int(0).into()],
                );
                candidate.init_bb.get_last_inst().insert_before(gep_zero);
                map.insert(*inst, gep_zero);
            }
        }
        replace_stack_reference(candidate.lo, &map)?;

        // Get current thread count
        let func_create = self
            .program
            .module
            .functions
            .iter()
            .find(|f| f.name == "thrd_create")
            .unwrap();
        let inst_create = self
            .program
            .mem_pool
            .get_call(*func_create, vec![Constant::Int(N_THREAD - 1).into()]);
        candidate.init_bb.get_last_inst().insert_before(inst_create);

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
        let i = candidate.init_val;
        let e = candidate.exit_val;

        // e - i
        let inst_sub = self.program.mem_pool.get_sub(e.clone(), i.clone());
        candidate.init_bb.get_last_inst().insert_before(inst_sub);

        // (e - i) * n
        let inst_mul = self
            .program
            .mem_pool
            .get_mul(inst_create.into(), inst_sub.into());
        candidate.init_bb.get_last_inst().insert_before(inst_mul);

        // (e - i) * n / N
        let inst_div = self
            .program
            .mem_pool
            .get_sdiv(inst_mul.into(), Constant::Int(N_THREAD).into());
        candidate.init_bb.get_last_inst().insert_before(inst_div);

        // Lower bound: i + (e - i) * n / N
        let inst_lb = self.program.mem_pool.get_add(inst_div.into(), i.clone());
        candidate.init_bb.get_last_inst().insert_before(inst_lb);

        // (e - i) * n + e - i
        let inst_add = self
            .program
            .mem_pool
            .get_add(inst_mul.into(), inst_sub.into());
        candidate.init_bb.get_last_inst().insert_before(inst_add);

        // ((e - i) * n + e - i) / N
        let inst_div = self
            .program
            .mem_pool
            .get_sdiv(inst_add.into(), Constant::Int(N_THREAD).into());
        candidate.init_bb.get_last_inst().insert_before(inst_div);

        // Upper bound: i + ((e - i) * n + e - i) / N
        let inst_ub = self.program.mem_pool.get_add(inst_div.into(), i);
        candidate.init_bb.get_last_inst().insert_before(inst_ub);

        // Replace indvar to parallelized indvar
        let phi = downcast_mut::<Phi>(candidate.indvar.as_mut().as_mut());
        phi.replace_incoming_value_at(candidate.init_bb, inst_lb.into());

        // Replace exit condition to parallelized exit condition
        let inst_cond = self.program.mem_pool.get_icmp(
            ICmpOp::Slt,
            ValueType::Int,
            candidate.indvar.into(),
            inst_ub.into(),
        );
        candidate.exit.insert_before(inst_cond);
        candidate.exit.set_operand(0, inst_cond.into());

        // Join threads
        let func_join = self
            .program
            .module
            .functions
            .iter()
            .find(|f| f.name == "thrd_join")
            .unwrap();
        let inst_join = self.program.mem_pool.get_call(*func_join, vec![]);
        candidate.exit_bb.push_front(inst_join);
        Ok(true)
    }
}

/// Get base pointer of load / store / gep instruction, return if it's alloc.
fn get_base_alloc(inst: InstPtr) -> Option<InstPtr> {
    if inst.get_type() == InstType::Alloca {
        return Some(inst);
    }
    match inst.get_type() {
        InstType::Load | InstType::GetElementPtr => {
            let ptr = inst.get_operand().first()?;
            if let Operand::Instruction(ptr) = ptr {
                get_base_alloc(*ptr)
            } else {
                None
            }
        }
        InstType::Store => {
            let base = inst.get_operand().get(1)?;
            if let Operand::Instruction(inst) = base {
                get_base_alloc(*inst)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Replace stack reference to copied global array address.
fn replace_stack_reference(lo: LoopPtr, map: &HashMap<InstPtr, InstPtr>) -> Result<()> {
    for bb in &lo.blocks {
        for inst in bb.iter() {
            replace_base_alloc(inst, map);
        }
    }
    for sub_loop in lo.sub_loops.iter() {
        replace_stack_reference(*sub_loop, map)?;
    }
    Ok(())
}

/// Replace base pointer of load / store / gep instruction, return if it's alloc.
fn replace_base_alloc(mut inst: InstPtr, map: &HashMap<InstPtr, InstPtr>) {
    match inst.get_type() {
        InstType::Load => {
            let ptr = inst.get_operand().first().unwrap();
            if let Operand::Instruction(ptr) = ptr.clone() {
                if ptr.get_type() == InstType::Alloca {
                    inst.set_operand(0, map[&ptr].into());
                } else {
                    replace_base_alloc(ptr, map);
                }
            }
        }
        InstType::GetElementPtr => {
            if inst.get_operand().len() == 2 {
                // Refuse to replace `getelementptr %ptr, 0`
                return;
            }
            let ptr = inst.get_operand().first().unwrap();
            if let Operand::Instruction(ptr) = ptr.clone() {
                if ptr.get_type() == InstType::Alloca {
                    inst.set_operand(0, map[&ptr].into());
                } else {
                    replace_base_alloc(ptr, map);
                }
            }
        }
        InstType::Store => {
            let base = inst.get_operand().get(1).unwrap();
            if let Operand::Instruction(ptr) = base.clone() {
                if ptr.get_type() == InstType::Alloca {
                    inst.set_operand(1, map[&ptr].into());
                } else {
                    replace_base_alloc(ptr, map);
                }
            }
        }
        _ => (),
    }
}

/// Merge effect if parallelizing them doesn't cause collision.
/// Returns changed or not.
fn merge_effect(a: &mut Effect, b: &Effect, indvar: &Operand) -> Result<bool> {
    if a.def_range.can_conflict(&b.def_range, indvar)
        || a.use_range.can_conflict(&b.def_range, indvar)
        || a.def_range.can_conflict(&b.use_range, indvar)
        || b.def_range.can_conflict(&b.use_range, indvar)
        || b.def_range.can_conflict(&b.def_range, indvar)
    {
        println!(
            "[INFO] failed to merge {} with {} (indvar = {})",
            a.dump(),
            b.dump(),
            indvar
        );
        return Ok(false);
    }
    a.def_range.merge(&b.def_range);
    a.use_range.merge(&b.use_range);
    Ok(true)
}

/// A candidate for parallelization.
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
/// exit_val = 6
struct Candidate {
    lo: LoopPtr,
    indvar: InstPtr,
    exit: InstPtr,
    init_val: Operand,
    init_bb: BBPtr,
    exit_val: Operand,
    exit_bb: BBPtr,
}

impl Candidate {
    fn new(
        lo: LoopPtr,
        indvar: InstPtr,
        exit: InstPtr,
        init_val: Operand,
        init_bb: BBPtr,
        exit_val: Operand,
        exit_bb: BBPtr,
    ) -> Self {
        Self {
            lo,
            indvar,
            exit,
            init_val,
            init_bb,
            exit_val,
            exit_bb,
        }
    }

    /// Dump candidate to string for debugging.
    fn dump(&self) -> String {
        format!(
            "Candidate {{\n  indvar: {},\n  exit: {},\n  init_val: {},\n  init_bb: {},\n  exit_val: {},\n  exit_bb: {},\n}}",
            self.indvar.gen_llvm_ir(),
            self.exit.gen_llvm_ir(),
            self.init_val,
            self.init_bb.name,
            self.exit_val,
            self.exit_bb.name,
        )
    }

    /// Get induction variable from exit instruction.
    /// Exit instruction should shape like:
    /// `exit = br (indvar < N), loop, exit`
    fn from_exit(exit: InstPtr, lo: LoopPtr, dom_tree: &mut DominatorTree) -> Option<Self> {
        let pre_header = lo.pre_header.unwrap();
        if exit.get_type() != InstType::Br {
            println!(
                "[INFO] loop {} fails because {} is not br",
                pre_header.name,
                exit.gen_llvm_ir()
            );
            return None;
        }

        // Get basic block to go to when exit
        let parent_bb = exit.get_parent_bb()?;
        let exit_bb = parent_bb
            .get_succ_bb()
            .iter()
            .find(|bb| !lo.is_in_loop(bb))?;

        // Exit block should have only one pred
        // TODO-PERF: this is for easy thread join implementation, but weakens optimization
        if exit_bb.get_pred_bb().len() != 1 {
            println!(
                "[INFO] loop {} fails because {} has multiple preds",
                pre_header.name, exit_bb.name
            );
            return None;
        }

        // Condition should be `indvar < op`, get `indvar` from condition
        // TODO-PERF: use induction variable analysis to get `indvar` consistently
        let Operand::Instruction(cond) = exit.get_operand().first()? else {
            println!(
                "[INFO] loop {} fails because {}'s first operand is not inst",
                pre_header.name,
                exit.gen_llvm_ir()
            );
            return None;
        };
        let InstType::ICmp = cond.get_type() else {
            println!(
                "[INFO] loop {} fails because {} is not condition",
                pre_header.name,
                cond.gen_llvm_ir()
            );
            return None;
        };
        let icmp = downcast_ref::<ICmp>(cond.as_ref().as_ref());
        if icmp.op != ICmpOp::Slt {
            println!(
                "[INFO] loop {} fails because {} is not slt",
                pre_header.name,
                cond.gen_llvm_ir()
            );
            return None;
        }
        let Operand::Instruction(indvar) = icmp.get_lhs() else {
            println!(
                "[INFO] loop {} fails because {}'s lhs is not inst",
                pre_header.name,
                cond.gen_llvm_ir()
            );
            return None;
        };
        let exit_val = icmp.get_rhs().clone();

        // Exit val should be calculated before loop (dominates pre_header)
        if let Operand::Instruction(inst) = exit_val {
            if !dom_tree.is_dominate(inst.get_parent_bb().unwrap(), pre_header) {
                println!(
                    "[INFO] loop {} fails because {} is not calculated before loop",
                    pre_header.name,
                    inst.gen_llvm_ir()
                );
                return None;
            }
        }

        // Indvar should be `phi [init_val, init_bb], [indvar + delta, next_bb]`
        // `indvar` should be the only phi in its block (other phi can be non-trivial)
        // `init_bb` should be `pre_header`
        // `next_bb` should be in loop
        if indvar.get_type() != InstType::Phi {
            println!(
                "[INFO] loop {} fails because {} is not phi",
                pre_header.name,
                indvar.gen_llvm_ir()
            );
            return None;
        }
        let phi = downcast_ref::<Phi>(indvar.as_ref().as_ref());
        let inc = phi.get_incoming_values();
        if inc.len() != 2 {
            println!(
                "[INFO] loop {} fails because {}'s incoming value length is not 2",
                pre_header.name,
                indvar.gen_llvm_ir()
            );
            return None;
        }
        let init_val = inc[0].0.clone();
        let init_bb = lo.pre_header?;
        if init_bb != inc[0].1 {
            println!(
                "[INFO] loop {} fails because {} is not pre_header",
                pre_header.name,
                inc[0].1.name.clone()
            );
            return None;
        }
        let Operand::Instruction(next_val) = inc[1].0 else {
            println!(
                "[INFO] loop {} fails because {}'s second incoming value is not inst",
                pre_header.name,
                indvar.gen_llvm_ir()
            );
            return None;
        };
        if next_val.get_type() != InstType::Add {
            println!(
                "[INFO] loop {} fails because {} is not add",
                pre_header.name,
                next_val.gen_llvm_ir()
            );
            return None;
        }
        let Operand::Constant(_) = next_val.get_operand()[1] else {
            println!(
                "[INFO] loop {} fails because {}'s second operand is not constant",
                pre_header.name,
                next_val.gen_llvm_ir()
            );
            return None;
        };
        let next_bb = inc[1].1;
        if !lo.is_in_loop(&next_bb) {
            println!(
                "[INFO] loop {} fails because {} is not in loop",
                pre_header.name, next_bb.name
            );
            return None;
        }
        for inst in indvar.get_parent_bb().unwrap().iter() {
            if inst.get_type() == InstType::Phi && inst != *indvar {
                println!(
                    "[INFO] loop {} fails because {} has multiple phi",
                    pre_header.name,
                    indvar.get_parent_bb().unwrap().name
                );
                return None;
            }
        }

        // Construct induction variable
        Some(Self::new(
            lo, *indvar, exit, init_val, init_bb, exit_val, *exit_bb,
        ))
    }
}
