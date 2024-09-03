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

use super::*;

impl_conversion_inst!(I2fInst, "fcvt.s.w");
impl_conversion_inst!(F2iInst, "fcvt.w.s", "rtz");

// impl conversion to Inst
impl_inst_convert!(I2fInst, I2f);
impl_inst_convert!(F2iInst, F2i);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_i2f_inst() {
        let inst = I2fInst::new(REG_FA0.into(), REG_A0.into());
        assert_eq!(inst.gen_asm(), "fcvt.s.w fa0,a0");
    }
    #[test]
    fn test_f2i_inst() {
        let inst = F2iInst::new(REG_A0.into(), REG_FA0.into());
        assert_eq!(inst.gen_asm(), "fcvt.w.s a0,fa0,rtz");
    }
}
