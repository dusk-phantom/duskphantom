use std::time::Instant;

use anyhow::Result;

pub mod block_fuse;
pub mod constant_fold;
pub mod dead_code_elim;
pub mod func_inline;
pub mod inst_combine;
pub mod ldce;
pub mod licm;
pub mod load_elim;
pub mod load_store_elim;
pub mod loop_optimization;
pub mod loop_simplify;
pub mod mem2reg;
pub mod redundance_elim;
pub mod sink;
pub mod store_elim;
pub mod ultimate_pass;

pub trait Transform {
    fn name() -> String;

    fn run(&mut self) -> Result<bool>;

    fn run_and_log(&mut self) -> Result<bool> {
        let before_run = Instant::now();
        let changed = self.run()?;
        let elapsed = before_run.elapsed().as_millis();
        println!(
            "{}: elapsed = {} ms {}",
            Self::name(),
            elapsed,
            if changed { "(changed)" } else { "" }
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
