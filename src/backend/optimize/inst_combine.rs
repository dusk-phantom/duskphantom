use super::*;
/// 处理指令结合,一些指令的组合可能被优化成一条指令
pub fn handle_inst_combine(func: &mut Func) -> Result<()> {
    // FIXME
    Func::combine_for_gep(func)?;
    Func::rm_useless_def_reg(func)?;

    Ok(())
}
impl Func {
    pub fn combine_for_gep(func: &mut Func) -> Result<()> {
        func.iter_bbs_mut().try_for_each(Block::combine_for_gep)
    }

    pub fn rm_useless_def_reg(func: &mut Func) -> Result<()> {
        let reg_lives = Func::reg_lives(func)?;
        func.iter_bbs_mut()
            .try_for_each(|bb| Block::rm_useless_def_reg(bb, reg_lives.live_outs(bb)))
    }
}
impl Block {
    /// FIXME
    ///
    /// this function should be call in abstract asmbly stage
    pub fn combine_for_gep(block: &mut Block) -> Result<()> {
        // 主要处理指令:add,sll,sw,lw
        let mut reg_imms: HashMap<Reg, Imm> = HashMap::new();

        let get_imm = |op: &Operand, reg_vals: &HashMap<Reg, Imm>| -> Option<Imm> {
            if let Operand::Imm(imm) = op {
                Some(imm.clone())
            } else if let Operand::Reg(reg) = op {
                reg_vals.get(reg).cloned()
            } else {
                None
            }
        };
        for inst in block.insts_mut().iter_mut() {
            // replace uses
            match inst {
                Inst::Add(add) => {
                    if let Some(rhs) = get_imm(add.rhs(), &reg_imms) {
                        *add.rhs_mut() = Operand::Imm(rhs);
                    }
                }
                Inst::Mul(mul) => {
                    if let Some(rhs) = get_imm(mul.rhs(), &reg_imms) {
                        *mul.rhs_mut() = Operand::Imm(rhs);
                    }
                }
                _ => {}
            }
            // refresh defs
            let dst_val: Option<Imm> = match inst {
                Inst::Add(add) => {
                    if let (Some(lhs), Some(rhs)) =
                        (get_imm(add.lhs(), &reg_imms), get_imm(add.rhs(), &reg_imms))
                    {
                        let v: Imm = (lhs + rhs);
                        Some(v)
                    } else {
                        None
                    }
                }
                Inst::Sll(sll) => {
                    if let (Some(lhs), Some(rhs)) =
                        (get_imm(sll.lhs(), &reg_imms), get_imm(sll.rhs(), &reg_imms))
                    {
                        let v: Imm = (lhs << rhs.try_into()?);
                        Some(v)
                    } else {
                        None
                    }
                }
                Inst::Mul(mul) => {
                    if let (Some(lhs), Some(rhs)) =
                        (get_imm(mul.lhs(), &reg_imms), get_imm(mul.rhs(), &reg_imms))
                    {
                        let v: Imm = (lhs * rhs);
                        Some(v)
                    } else {
                        None
                    }
                }
                Inst::Mv(mv) => get_imm(mv.src(), &reg_imms),
                Inst::Li(li) => Some(li.src().try_into()?),
                _ => None,
            };
            if let Some(dst_var) = dst_val {
                assert_eq!(inst.defs().len(), 1);
                let dst = inst.defs().first().cloned().unwrap();
                reg_imms.insert(*dst, dst_var.clone());
                *inst = LiInst::new(dst.into(), dst_var.into()).into();
            } else {
                for reg in inst.defs() {
                    reg_imms.remove(reg);
                }
            }
        }
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
                    alive_regs.retain(|reg| !inst.defs().contains(&reg));
                    alive_regs.extend(inst.uses().iter().cloned());
                } else {
                    is_changed = true;
                }
            }
            let mut new_insts = new_insts_rev.into_iter().rev().collect();
            *bb.insts_mut() = new_insts;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1;

    use insta::assert_snapshot;

    use crate::utils::diff::diff;

    use super::*;
    #[test]
    fn test_combine_for_gep() {
        // li x32,0
        // li x33,4
        // slliw x34,x32,1
        // slliw x40,x32,3
        // addw x34,x34,x40
        // add x35,x33,x34
        // slli x36,x35,2
        // load_addr x37,[0-40]
        // add x38,x37,x36
        // lw x39,0(x38)
        // FIXME
        let mut bb = Block::new("test".to_string());
        let mut ssa = StackAllocator::new();
        let x32 = Reg::new(32, true);
        let x33 = Reg::new(33, true);
        let x34 = Reg::new(34, true);
        let x35 = Reg::new(35, true);
        let x36 = Reg::new(36, true);
        let x37 = Reg::new(37, true);
        let x38 = Reg::new(38, true);
        let x39 = Reg::new(39, true);
        let x40 = Reg::new(40, true);
        bb.push_inst(LiInst::new(x32.into(), 0.into()).into());
        bb.push_inst(LiInst::new(x33.into(), 4.into()).into());
        bb.push_inst(SllInst::new(x34.into(), x32.into(), 1.into()).into());
        bb.push_inst(SllInst::new(x40.into(), x32.into(), 3.into()).into());
        bb.push_inst(AddInst::new(x34.into(), x34.into(), x40.into()).into());
        bb.push_inst(AddInst::new(x35.into(), x33.into(), x34.into()).into());
        bb.push_inst(SllInst::new(x36.into(), x35.into(), 2.into()).into());
        bb.push_inst(LocalAddr::new(x37, ssa.alloc(40)).into());
        bb.push_inst(AddInst::new(x38.into(), x37.into(), x36.into()).into());
        bb.push_inst(LwInst::new(x39, 0.into(), x38).into());

        let asm_before = bb.gen_asm();
        Block::combine_for_gep(&mut bb).unwrap();
        let asm_after = bb.gen_asm();
        assert_snapshot!(diff(&asm_before, &asm_after),@r###"
        test:
        li x32,0
        li x33,4
        [-] slliw x34,x32,1
        [-] slliw x40,x32,3
        [-] addw x34,x34,x40
        [-] addw x35,x33,x34
        [-] slliw x36,x35,2
        [+] li x34,0
        [+] li x40,0
        [+] li x34,0
        [+] li x35,4
        [+] li x36,16
        load_addr x37,[0-40]
        [-] addw x38,x37,x36
        [+] addiw x38,x37,16
        lw x39,0(x38)
        "###);
        Block::rm_useless_def_reg(&mut bb, &vec![x39].into_iter().collect()).unwrap();
        let asm_after2 = bb.gen_asm();
        assert_snapshot!(diff(&asm_after, &asm_after2),@r###"
        test:
        [-] li x32,0
        [-] li x33,4
        [-] li x34,0
        [-] li x40,0
        [-] li x34,0
        [-] li x35,4
        [-] li x36,16
        load_addr x37,[0-40]
        addiw x38,x37,16
        lw x39,0(x38)
        "###);
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
