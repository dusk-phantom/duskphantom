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

pub fn post_handle_inst_split(func: &mut Func) -> Result<()> {
    Func::post_split_li(func)?;
    Ok(())
}

impl Func {
    /// handle li , li
    fn post_split_li(func: &mut Func) -> Result<()> {
        func.iter_bbs_mut().try_for_each(Block::post_split_li)?;
        Ok(())
    }
}

impl Block {
    fn post_split_li(bb: &mut Block) -> Result<()> {
        let mut new_insts = Vec::new();
        for i in bb.insts_mut() {
            if let Inst::Li(li) = i {
                let imm = li.src().imm().ok_or_else(|| anyhow!(""))?;
                if imm.in_limit_12() {
                    new_insts.push(li.clone().into());
                } else {
                    let imm = *imm;
                    let dst = li.dst().reg().with_context(|| context!())?;
                    if (-(1 << 31)..(1 << 31)).contains(&imm) {
                        Block::post_split_li32(imm, dst, &mut new_insts)?;
                    } else if (-(1 << 43)..(1 << 43)).contains(&imm) {
                        Block::post_split_li44(imm, dst, &mut new_insts)?;
                    } else if (-(1 << 55)..(1 << 55)).contains(&imm) {
                        Block::post_split_li56(imm, dst, &mut new_insts)?;
                    } else {
                        Block::post_split_li64(imm, dst, &mut new_insts)?;
                    }
                }
            } else {
                // 其他类型的指令就直接穿过
                new_insts.push(i.clone());
            }
        }
        *bb.insts_mut() = new_insts;
        Ok(())
    }

    fn post_split_li32(imm: i64, dst: Reg, new_insts: &mut Vec<Inst>) -> Result<()> {
        let hi = ((imm + 0x0800) >> 12) & 0x000f_ffff; // 20
        let lo = (imm << 52) >> 52; // 12
        if lo == 0 {
            let lui = LuiInst::new(dst.into(), hi.into());
            new_insts.push(lui.into());
        } else {
            let lui = LuiInst::new(dst.into(), hi.into());
            let addi = AddInst::new(dst.into(), dst.into(), lo.into()).with_8byte();
            new_insts.push(lui.into());
            new_insts.push(addi.into());
        }
        Ok(())
    }
    fn post_split_li44(imm: i64, dst: Reg, new_insts: &mut Vec<Inst>) -> Result<()> {
        let hi = (imm + 0x0800) >> 12; // 32
        let lo = (imm << 52) >> 52; // 12
        Block::post_split_li32(hi, dst, new_insts)?;
        let slli = SllInst::new(dst.into(), dst.into(), (12).into()).with_8byte();
        new_insts.push(slli.into());
        if lo != 0 {
            let addi = AddInst::new(dst.into(), dst.into(), lo.into()).with_8byte();
            new_insts.push(addi.into());
        }
        Ok(())
    }
    fn post_split_li56(imm: i64, dst: Reg, new_insts: &mut Vec<Inst>) -> Result<()> {
        let hi = (imm + 0x0800) >> 12; // 44
        let lo = (imm << 52) >> 52; // 12
        Block::post_split_li44(hi, dst, new_insts)?;
        let slli = SllInst::new(dst.into(), dst.into(), (12).into()).with_8byte();
        new_insts.push(slli.into());
        if lo != 0 {
            let addi = AddInst::new(dst.into(), dst.into(), lo.into()).with_8byte();
            new_insts.push(addi.into());
        }
        Ok(())
    }
    fn post_split_li64(imm: i64, dst: Reg, new_insts: &mut Vec<Inst>) -> Result<()> {
        let hi = (imm + 0x0080) >> 8; // 56
        let lo = (imm << 56) >> 56; // 8
        Block::post_split_li56(hi, dst, new_insts)?;
        let slli = SllInst::new(dst.into(), dst.into(), (8).into()).with_8byte();
        new_insts.push(slli.into());
        if lo != 0 {
            let addi = AddInst::new(dst.into(), dst.into(), lo.into()).with_8byte();
            new_insts.push(addi.into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused)]
    use insta::assert_snapshot;

    #[allow(unused)]
    use super::*;

    use crate::utils::diff::diff;

    #[test]
    fn test_post_split_li() {
        let mut bb0 = Block::new("bb0".to_string());
        let mut bb1 = Block::new("bb1".to_string());
        bb0.push_inst(LiInst::new(REG_A0.into(), (31415926).into()).into());
        bb1.push_inst(LiInst::new(REG_A0.into(), (31415926).into()).into());
        Block::post_split_li(&mut bb0).unwrap();
        let bb0_asm = bb0.gen_asm();
        let bb1_asm = bb1.gen_asm();
        // 7670 << 12 = 31416320
        // 31416320 - 394 = 31415926
        assert_snapshot!(diff(&bb0_asm, &bb1_asm), @r###"
        [-] bb0:
        [-] lui a0,7670
        [-] addi a0,a0,-394
        [+] bb1:
        [+] li a0,31415926
        "###);

        let mut bb0 = Block::new("bb0".to_string());
        let mut bb1 = Block::new("bb1".to_string());
        bb0.push_inst(LiInst::new(REG_A0.into(), (31415926535).into()).into());
        bb1.push_inst(LiInst::new(REG_A0.into(), (31415926535).into()).into());
        Block::post_split_li(&mut bb0).unwrap();
        let bb0_asm = bb0.gen_asm();
        let bb1_asm = bb1.gen_asm();
        // 1873 << 12 = 7671808
        // 7671808 - 249 = 7669904
        // 7669904 << 12 = 31415926784
        // 31415926784 - 249 = 31415926535
        assert_snapshot!(diff(&bb0_asm, &bb1_asm), @r###"
        [-] bb0:
        [-] lui a0,1873
        [-] addi a0,a0,-1904
        [-] slli a0,a0,12
        [-] addi a0,a0,-249
        [+] bb1:
        [+] li a0,31415926535
        "###);

        let mut bb0 = Block::new("bb0".to_string());
        let mut bb1 = Block::new("bb1".to_string());
        bb0.push_inst(LiInst::new(REG_A0.into(), (3141592653589793).into()).into());
        bb1.push_inst(LiInst::new(REG_A0.into(), (3141592653589793).into()).into());
        Block::post_split_li(&mut bb0).unwrap();
        let bb0_asm = bb0.gen_asm();
        let bb1_asm = bb1.gen_asm();
        // 45716 << 12 = 187252736
        // 187252736 + 776 = 187253514
        // 187253514 << 12 = 766990393344
        // 766990393344 + 599 = 766990393943
        // 766990393943 << 12 = 3141592653590528
        // 3141592653590528 - 735 = 3141592653589793
        assert_snapshot!(diff(&bb0_asm, &bb1_asm), @r###"
        [-] bb0:
        [-] lui a0,45716
        [-] addi a0,a0,778
        [-] slli a0,a0,12
        [-] addi a0,a0,599
        [-] slli a0,a0,12
        [-] addi a0,a0,-735
        [+] bb1:
        [+] li a0,3141592653589793
        "###);

        let mut bb0 = Block::new("bb0".to_string());
        let mut bb1 = Block::new("bb1".to_string());
        bb0.push_inst(LiInst::new(REG_A0.into(), (314159265358979323).into()).into());
        bb1.push_inst(LiInst::new(REG_A0.into(), (314159265358979323).into()).into());
        Block::post_split_li(&mut bb0).unwrap();
        let bb0_asm = bb0.gen_asm();
        let bb1_asm = bb1.gen_asm();
        // 17858 << 12 = 73146368
        // 73146368 - 464 = 73145904
        // 73145904 << 12 = 299605622784
        // 299605622784 - 150 = 299605622634
        // 299605622634 << 12 = 1227184630308864
        // 1227184630308864 - 351 = 1227184630308513
        // 1227184630308513 << 8 = 314159265358979328
        // 314159265358979328 - 5 = 314159265358979323
        assert_snapshot!(diff(&bb0_asm, &bb1_asm), @r###"
        [-] bb0:
        [-] lui a0,17858
        [-] addi a0,a0,-464
        [-] slli a0,a0,12
        [-] addi a0,a0,-150
        [-] slli a0,a0,12
        [-] addi a0,a0,-351
        [-] slli a0,a0,8
        [-] addi a0,a0,-5
        [+] bb1:
        [+] li a0,314159265358979323
        "###);
    }
}
