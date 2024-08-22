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

#[allow(unused)]
use super::*;

#[cfg(test)]
mod test_reg_def_use {
    use super::*;
    #[test]
    fn test_seqz() {
        let mut reg_gener = RegGenerator::new();
        let dst = reg_gener.gen_virtual_usual_reg();
        let src = reg_gener.gen_virtual_usual_reg();
        let seqz = SeqzInst::new(dst.into(), src.into());
        assert_eq!(seqz.uses(), vec![&src]);
        assert_eq!(seqz.defs(), vec![&dst]);
    }
    #[test]
    fn test_repeat() {
        let mul = MulInst::new(REG_A0.into(), REG_A0.into(), REG_A0.into());
        assert_eq!(mul.uses(), vec![&REG_A0]);
        assert_eq!(mul.defs(), vec![&REG_A0]);
        let beq = BeqInst::new(REG_A0, REG_A0, "a".into());
        assert_eq!(beq.uses(), vec![&REG_A0]);
        let beq2 = BeqInst::new(REG_A0, REG_A1, "a".into());
        assert_eq!(beq2.uses(), vec![&REG_A0, &REG_A1]);
    }
    #[test]
    fn test_mv() {
        let mv = MvInst::new(REG_A0.into(), REG_A1.into());
        assert_eq!(mv.uses(), vec![&REG_A1]);
        assert_eq!(mv.defs(), vec![&REG_A0]);
    }
    #[test]
    fn test_local_addr() {
        let mut ssa = StackAllocator::new();
        let l = LocalAddr::new(REG_A0, ssa.alloc(80));
        assert!(l.uses().is_empty());
        assert_eq!(l.defs(), vec![&REG_A0]);
    }
}

#[cfg(test)]
mod test_reg_replace {
    use super::*;

    use super::irs::{NotInst, Reg, RegGenerator, RegReplace, REG_A1};

    #[test]
    fn test_not() {
        let mut reg_genner = RegGenerator::new();
        let r1 = reg_genner.gen_virtual_usual_reg();
        let r2 = reg_genner.gen_virtual_usual_reg();
        let mut not = NotInst::new(r1.into(), r2.into());
        not.replace_use(r2, REG_A1).unwrap();
        assert_eq!(not.uses(), vec![&REG_A1]);
        not.replace_def(r1, REG_A0).unwrap();
        assert_eq!(not.defs(), vec![&REG_A0]);
    }

    #[test]
    fn test_not_2() {
        let dst = Reg::new(40, true);
        let src = Reg::new(5, true);
        let mut not = NotInst::new(dst.into(), src.into());
        dbg!(&not);
        not.replace_def(dst, Reg::new(6, true)).unwrap();
        dbg!(&not);
    }

    #[test]
    fn test_local_addr() {
        let mut ssa = StackAllocator::new();
        let mut l = LocalAddr::new(REG_A0, ssa.alloc(80));
        assert_eq!(l.defs(), vec![&REG_A0]);
        l.replace_def(REG_A0, REG_A1).unwrap();
        assert_eq!(l.defs(), vec![&REG_A1]);
    }
}
