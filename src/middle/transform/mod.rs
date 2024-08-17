use std::time::Instant;

use anyhow::Result;

use crate::utils::diff::diff;

use super::Program;

pub mod block_fuse;
pub mod constant_fold;
pub mod dead_code_elim;
pub mod func_inline;
pub mod inst_combine;
pub mod ldce;
pub mod licm;
pub mod load_elim;
pub mod load_store_elim;
pub mod loop_depth;
pub mod loop_optimization;
pub mod loop_simplify;
pub mod make_parallel;
pub mod mem2reg;
pub mod redundance_elim;
pub mod sink_code;
pub mod store_elim;
pub mod ultimate_pass;

pub trait Transform {
    fn name() -> String;

    fn get_program_mut(&mut self) -> &mut Program;

    fn run(&mut self) -> Result<bool>;

    fn run_and_log(&mut self) -> Result<bool> {
        let time_before = Instant::now();
        let changed = self.run()?;
        let elapsed = time_before.elapsed().as_micros();
        println!(
            "## Pass {} {}\n\nTime elapsed = {} µs\n",
            Self::name(),
            if changed { "[CHANGED]" } else { "" },
            elapsed,
        );
        Ok(changed)
    }

    #[allow(unused)]
    fn run_and_debug(&mut self) -> Result<bool> {
        let time_before = Instant::now();
        let program_before = self.get_program_mut().module.gen_llvm_ir();
        let changed = self.run()?;
        let elapsed = time_before.elapsed().as_micros();
        let program_after = self.get_program_mut().module.gen_llvm_ir();
        println!(
            "## Pass {}\n\nTime elapsed = {} µs\n\nDiff:\n\n```diff\n{}```\n",
            Self::name(),
            elapsed,
            diff(&program_before, &program_after)
        );
        Ok(changed)
    }

    fn loop_and_log(&mut self) -> Result<bool> {
        let mut changed = false;
        loop {
            let c = self.run_and_log()?;
            changed |= c;
            if !c {
                break;
            }
        }
        Ok(changed)
    }
}
