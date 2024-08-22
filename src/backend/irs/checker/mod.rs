// Copyright 2024 Duskphantom Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

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
