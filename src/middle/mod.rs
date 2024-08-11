use crate::{
    /* errors::MiddleError, */ frontend,
    utils::{mem::ObjPtr, paral_counter::ParalCounter},
};
use analysis::{call_graph::CallGraph, effect_analysis::EffectAnalysis, memory_ssa::MemorySSA};
use anyhow::Context;
use ir::ir_builder::IRBuilder;
use transform::{
    block_fuse, constant_fold, dead_code_elim, func_inline, inst_combine, load_elim,
    loop_optimization, mem2reg, simple_gvn, store_elim, unreachable_block_elim,
};

pub mod analysis;
pub mod ir;
pub mod irgen;
pub mod transform;

use std::pin::Pin;

pub struct Program {
    pub module: ir::Module,
    pub mem_pool: Pin<Box<IRBuilder>>,
}

use crate::context;
use anyhow::Result;

pub fn gen(program: &frontend::Program) -> Result<Program> {
    irgen::gen(program).with_context(|| context!())
    // match irgen::gen(program) {
    //     Ok(program) => Ok(program),
    //     Err(_) => Err(MiddleError::GenError),
    // }
}

pub fn optimize(program: &mut Program) {
    // Convert program to SSA and inline functions
    let mut call_graph = CallGraph::new(program);
    let counter = ParalCounter::new(0, usize::MAX);
    mem2reg::optimize_program(program).unwrap();
    func_inline::optimize_program(program, &mut call_graph, counter).unwrap();
    dead_code_elim::optimize_program(program).unwrap();

    // Further optimize
    for _ in 0..3 {
        // Weaken instructions
        constant_fold::optimize_program(program).unwrap();
        inst_combine::optimize_program(program).unwrap();
        simple_gvn::optimize_program(program).unwrap();

        // Remove unused code
        let effect_analysis = EffectAnalysis::new(program);
        let mut memory_ssa = MemorySSA::new(program, &effect_analysis);
        load_elim::optimize_program(program, &mut memory_ssa).unwrap();
        store_elim::optimize_program(program, &mut memory_ssa, &effect_analysis).unwrap();

        // Remove unreachable block and instruction
        unreachable_block_elim::optimize_program(program).unwrap();
        block_fuse::optimize_program(program).unwrap();
        dead_code_elim::optimize_program(program).unwrap();
    }

    let _ = std::fs::write("before.ll", program.module.gen_llvm_ir()).with_context(|| context!());
    // Loop optimization
    loop_optimization::optimize_program(program).unwrap();
    let _ = std::fs::write("after.ll", program.module.gen_llvm_ir()).with_context(|| context!());

    // Further optimize after loop optimization
    for _ in 0..3 {
        // Weaken instructions
        constant_fold::optimize_program(program).unwrap();
        inst_combine::optimize_program(program).unwrap();
        simple_gvn::optimize_program(program).unwrap();

        // Remove unused code
        let effect_analysis = EffectAnalysis::new(program);
        let mut memory_ssa = MemorySSA::new(program, &effect_analysis);
        load_elim::optimize_program(program, &mut memory_ssa).unwrap();
        store_elim::optimize_program(program, &mut memory_ssa, &effect_analysis).unwrap();

        // Remove unreachable block and instruction
        unreachable_block_elim::optimize_program(program).unwrap();
        block_fuse::optimize_program(program).unwrap();
        dead_code_elim::optimize_program(program).unwrap();
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

impl Program {
    pub fn new() -> Self {
        let program_mem_pool = Box::pin(IRBuilder::new());
        let mem_pool: ObjPtr<IRBuilder> = ObjPtr::new(&program_mem_pool);
        Self {
            mem_pool: program_mem_pool,
            module: ir::Module::new(mem_pool),
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        self.mem_pool.clear();
    }
}
