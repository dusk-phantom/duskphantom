use super::*;
/// 处理指令结合,一些指令的组合可能被优化成一条指令
pub fn handle_inst_combine(func: &mut Func) -> Result<()> {
    // FIXME
    Func::rm_useless_def_reg(func)?;
    // Func::combine_for_gep(func)?;
    Ok(())
}
impl Func {
    pub fn combine_for_gep(func: &mut Func) -> Result<()> {
        let reg_lives = Func::reg_lives(func)?;
        func.iter_bbs_mut()
            .try_for_each(|bb| Block::combine_for_gep(bb, reg_lives.live_outs(bb)))
    }

    pub fn rm_useless_def_reg(func: &mut Func) -> Result<()> {
        let reg_lives = Func::reg_lives(func)?;
        func.iter_bbs_mut()
            .try_for_each(|bb| Block::rm_useless_def_reg(bb, reg_lives.live_outs(bb)))
    }
}
impl Block {
    /// FIXME
    pub fn combine_for_gep(block: &mut Block, live_out: &HashSet<Reg>) -> Result<()> {
        // 主要处理指令:add,sll,sw,lw
        Ok(())
    }

    pub fn rm_useless_def_reg(bb: &mut Block, live_out: &HashSet<Reg>) -> Result<()> {
        let mut is_changed = true;
        while is_changed {
            is_changed = false;
            let mut new_insts_rev = Vec::new();
            let mut alive_regs = live_out.clone();
            for inst in bb.insts_mut().iter().rev() {
                if inst.is_control_flow() || inst.defs().iter().all(|reg| alive_regs.contains(reg))
                {
                    new_insts_rev.push(inst.clone());
                } else {
                    is_changed = true;
                }
                alive_regs.retain(|reg| !inst.defs().contains(&reg));
                alive_regs.extend(inst.uses().iter().cloned());
            }
            let mut new_insts = new_insts_rev.into_iter().rev().collect();
            *bb.insts_mut() = new_insts;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::utils::diff::diff;

    use super::*;
    #[test]
    fn test_combine_for_gep() {
        // FIXME
        // li a0,0
        // li s1,0
        // li a2,0
        // slliw a1,s1,1
        // add a4,a2,a1
        // slliw a1,a0,3
        // add s1,a4,a1
        // slli a1,s1,2
        // addi s1,s0,-96
        // add a0,s1,a1
        // li s1,1
        // sw s1,0(a0)
        // let mut bb = Block::new("test".to_string());
        // *bb.insts_mut() = vec![
        //     LiInst::new(REG_A0.into(), 0.into()).into(),
        //     LiInst::new(REG_S1.into(), 0.into()).into(),
        //     LiInst::new(REG_A2.into(), 0.into()).into(),
        //     SllInst::new(REG_A1.into(), REG_S1.into(), 1.into()).into(),
        //     AddInst::new(REG_A4.into(), REG_A2.into(), REG_A1.into()).into(),
        //     SllInst::new(REG_A1.into(), REG_A0.into(), 3.into()).into(),
        //     AddInst::new(REG_S1.into(), REG_A4.into(), REG_A1.into()).into(),
        //     SllInst::new(REG_A1.into(), REG_S1.into(), 2.into()).into(),
        //     AddInst::new(REG_S1.into(), REG_S0.into(), (-96).into()).into(),
        //     AddInst::new(REG_A0.into(), REG_S1.into(), REG_A1.into()).into(),
        //     LiInst::new(REG_S1.into(), 1.into()).into(),
        //     SwInst::new(REG_S1, 0.into(), REG_A0).into(),
        // ];
        // let asm_before = bb.gen_asm();
        // Block::combine_for_gep(&mut bb, &HashSet::new()).unwrap();
        // let asm_after = bb.gen_asm();
        // assert_snapshot!(diff(&asm_before, &asm_after),@r###""###);
    }

    #[test]
    fn test_remove_useless_def_reg() {
        let mut bb = Block::new("t".to_string());
        *bb.insts_mut() = vec![
            LiInst::new(REG_A0.into(), 0.into()).into(),
            LiInst::new(REG_S1.into(), 0.into()).into(),
        ];
        let live_out: HashSet<Reg> = vec![REG_S1].into_iter().collect();
        let old_asm = bb.gen_asm();
        Block::rm_useless_def_reg(&mut bb, &live_out).unwrap();
        let new_asm = bb.gen_asm();
        let diff = diff(&old_asm, &new_asm);
        assert_snapshot!(diff, @r###"
        t:
        [-] li a0,0
        li s1,0
        "###);
    }
    #[test]
    fn test_remove_useless_def_reg2() {
        let mut bb = Block::new("t".to_string());
        *bb.insts_mut() = vec![
            LiInst::new(REG_A0.into(), 0.into()).into(),
            LiInst::new(REG_S1.into(), 0.into()).into(),
            AddInst::new(REG_S1.into(), REG_A0.into(), REG_A1.into()).into(),
        ];
        let live_out: HashSet<Reg> = vec![REG_ZERO].into_iter().collect();
        let old_asm = bb.gen_asm();
        Block::rm_useless_def_reg(&mut bb, &live_out).unwrap();
        let new_asm = bb.gen_asm();
        let diff = diff(&old_asm, &new_asm);
        assert_snapshot!(diff, @r###"
        t:
        [-] li a0,0
        [-] li s1,0
        [-] addw s1,a0,a1
        [+] 
        "###);
    }
}
