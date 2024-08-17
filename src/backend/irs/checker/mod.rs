pub use super::*;

pub trait IRChecker: ProgramChecker {}

pub trait ProgramChecker: ModuleChecker {
    #[allow(unused)]
    fn check_prog(&self, program: &Program) -> bool {
        for mdl in &program.modules {
            if !self.check_mdl(mdl) {
                return false;
            }
        }
        true
    }
}

pub trait ModuleChecker: VarChecker + FuncChecker {
    #[allow(unused)]
    fn check_mdl(&self, module: &Module) -> bool {
        for var in module.global.iter() {
            if !VarChecker::check_var(self, var) {
                return false;
            }
        }
        for func in module.funcs.iter() {
            if !FuncChecker::check_func(self, func) {
                return false;
            }
        }
        true
    }
}

pub trait VarChecker {
    #[allow(unused)]
    fn check_var(&self, var: &Var) -> bool {
        false
    }
}

pub trait FuncChecker: BBChecker {
    fn check_func(&self, func: &Func) -> bool {
        for bb in func.iter_bbs() {
            if !self.check_bb(bb) {
                return false;
            }
        }
        true
    }
}

pub trait BBChecker: InstChecker {
    fn check_bb(&self, bb: &Block) -> bool {
        for inst in bb.insts() {
            if !self.check_inst(inst) {
                return false;
            }
        }
        true
    }
}
pub trait InstChecker {
    #[allow(unused)]
    fn check_inst(&self, inst: &Inst) -> bool {
        false
    }
}

mod riscv;
mod tight_term;

pub use riscv::*;
pub use tight_term::*;
