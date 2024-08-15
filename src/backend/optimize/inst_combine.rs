use rustc_hash::FxHashSet;

use super::*;
/// 处理指令结合,一些指令的组合可能被优化成一条指令
pub fn handle_inst_combine(func: &mut Func) -> Result<()> {
    Func::combine_for_gep(func)?;
    Func::rm_useless_def_reg(func)?;
    Func::combine_for_gep2(func)?;
    Func::rm_useless_def_reg(func)?;
    Func::combine_for_br(func)?;

    Ok(())
}
impl Func {
    pub fn combine_for_gep(func: &mut Func) -> Result<()> {
        func.iter_bbs_mut().try_for_each(Block::combine_for_gep)
    }
    pub fn combine_for_gep2(func: &mut Func) -> Result<()> {
        func.iter_bbs_mut().try_for_each(Block::combine_for_gep2)
    }

    pub fn rm_useless_def_reg(func: &mut Func) -> Result<()> {
        let reg_lives = Func::reg_lives(func)?;
        func.iter_bbs_mut()
            .try_for_each(|bb| Block::rm_useless_def_reg(bb, reg_lives.live_outs(bb)))
    }

    pub fn combine_for_br(func: &mut Func) -> Result<()> {
        func.iter_bbs_mut().try_for_each(Block::combine_for_br)
    }
}
impl Block {
    pub fn combine_for_br(block: &mut Block) -> Result<()> {
        // 倒数第二条指令是不是 beq <icmp>, REG_ZERO, <false_label>

        // 新指令序列
        let mut new_insts: Vec<Option<Inst>> = block
            .insts()
            .iter()
            .map(|inst| Some(inst.clone()))
            .collect();

        // 判断是不是 beq <icmp>, REG_ZERO, <label>
        if block.insts().len() < 2 {
            return Ok(());
        }
        let beq_idx = block.insts().len() - 2;
        if let Some(Inst::Beq(beq)) = block.insts().iter().rev().nth(1) {
            if beq.rhs().eq(&REG_ZERO) {
                // (reg, defined inst idx)
                let mut defs: HashMap<&Reg, usize> = HashMap::new();
                for (idx, inst) in block.insts().iter().enumerate() {
                    for reg in inst.defs() {
                        defs.insert(reg, idx);
                    }
                }
                let defs = defs;

                // lhs 定义在了其他基本块中
                let Some(icmp_def_idx) = defs.get(beq.lhs()) else {
                    return Ok(());
                };

                // 两种情况: slt/xor <- snez, seqz
                match block
                    .insts()
                    .get(*icmp_def_idx)
                    .with_context(|| context!())?
                {
                    Inst::Snez(snez) => {
                        let snez_idx = icmp_def_idx;
                        // snez 的 src 只能是 reg
                        let snez_src = snez.src().reg().with_context(|| context!())?;
                        if let Some(def_idx) = defs.get(&snez_src) {
                            // xor/slt
                            match block.insts().get(*def_idx).with_context(|| context!())? {
                                // xor -> snez -> beq
                                Inst::Xor(xor) => {
                                    let xor_idx = def_idx;
                                    new_insts[*xor_idx] = None;
                                    let lhs = xor.lhs().reg().with_context(|| context!())?;
                                    let label = beq.label().clone();
                                    match xor.rhs() {
                                        Operand::Reg(rhs) => {
                                            new_insts[*snez_idx] = None;
                                            let rhs = *rhs;
                                            new_insts[beq_idx] =
                                                Some(BeqInst::new(lhs, rhs, label).into());
                                        }
                                        Operand::Imm(imm) => {
                                            // 重复利用这个 reg 和 槽
                                            let rhs =
                                                snez.dst().reg().with_context(|| context!())?;
                                            new_insts[*snez_idx] =
                                                Some(LiInst::new(rhs.into(), (*imm).into()).into());
                                            new_insts[beq_idx] =
                                                Some(BeqInst::new(lhs, rhs, label).into());
                                        }
                                        _ => unimplemented!(),
                                    }
                                }
                                Inst::Slt(slt) => {
                                    let slt_idx = def_idx;
                                    new_insts[*slt_idx] = None;
                                    let label = beq.label().clone();
                                    let lhs = slt.lhs().reg().with_context(|| context!())?;
                                    match slt.rhs() {
                                        Operand::Reg(rhs) => {
                                            let rhs = *rhs;
                                            new_insts[*snez_idx] = None;
                                            new_insts[beq_idx] =
                                                Some(BgeInst::new(lhs, rhs, label).into());
                                        }
                                        Operand::Imm(imm) => {
                                            // 重复利用这个 reg 和 槽
                                            let rhs =
                                                snez.dst().reg().with_context(|| context!())?;
                                            new_insts[*snez_idx] =
                                                Some(LiInst::new(rhs.into(), (*imm).into()).into());
                                            new_insts[beq_idx] =
                                                Some(BgeInst::new(lhs, rhs, label).into());
                                        }
                                        _ => unimplemented!(),
                                    }
                                }
                                _ => unimplemented!(),
                            }
                        } else {
                            // 可以合并 snez 与 beqz
                            let reg = snez.src().reg().with_context(|| context!())?;
                            let label = beq.label().clone();
                            new_insts[*snez_idx] = None;
                            new_insts[beq_idx] = Some(BeqInst::new(reg, REG_ZERO, label).into());
                        };
                    }
                    Inst::Seqz(seqz) => {
                        let seqz_idx = icmp_def_idx;
                        // snez 的 src 只能是 reg
                        let seqz_src = seqz.src().reg().with_context(|| context!())?;
                        if let Some(def_idx) = defs.get(&seqz_src) {
                            // xor/slt
                            match block.insts().get(*def_idx).with_context(|| context!())? {
                                // xor -> snez -> beq
                                Inst::Xor(xor) => {
                                    let xor_idx = def_idx;
                                    new_insts[*xor_idx] = None;
                                    let lhs = xor.lhs().reg().with_context(|| context!())?;
                                    let label = beq.label().clone();
                                    match xor.rhs() {
                                        Operand::Reg(rhs) => {
                                            new_insts[*seqz_idx] = None;
                                            let rhs = *rhs;
                                            new_insts[beq_idx] =
                                                Some(BneInst::new(lhs, rhs, label).into());
                                        }
                                        Operand::Imm(imm) => {
                                            // 重复利用这个 reg 和 槽
                                            let rhs =
                                                seqz.dst().reg().with_context(|| context!())?;
                                            new_insts[*seqz_idx] =
                                                Some(LiInst::new(rhs.into(), (*imm).into()).into());
                                            new_insts[beq_idx] =
                                                Some(BneInst::new(lhs, rhs, label).into());
                                        }
                                        _ => unimplemented!(),
                                    }
                                }
                                Inst::Slt(slt) => {
                                    let slt_idx = def_idx;
                                    new_insts[*slt_idx] = None;
                                    let label = beq.label().clone();
                                    let lhs = slt.lhs().reg().with_context(|| context!())?;
                                    match slt.rhs() {
                                        Operand::Reg(rhs) => {
                                            let rhs = *rhs;
                                            new_insts[*seqz_idx] = None;
                                            new_insts[beq_idx] =
                                                Some(BltInst::new(lhs, rhs, label).into());
                                        }
                                        Operand::Imm(imm) => {
                                            // 重复利用这个 reg 和 槽
                                            let rhs =
                                                seqz.dst().reg().with_context(|| context!())?;
                                            new_insts[*seqz_idx] =
                                                Some(LiInst::new(rhs.into(), (*imm).into()).into());
                                            new_insts[beq_idx] =
                                                Some(BltInst::new(lhs, rhs, label).into());
                                        }
                                        _ => unimplemented!(),
                                    }
                                }
                                Inst::Sltu(_) => todo!(),
                                Inst::Sgtu(_) => todo!(),
                                Inst::Seqz(_) => todo!(),
                                Inst::Snez(_) => todo!(),
                                Inst::Feqs(_) => todo!(),
                                Inst::Fles(_) => todo!(),
                                Inst::Flts(_) => todo!(),
                                _ => unimplemented!(),
                            }
                        } else {
                            // 可以合并 snez 与 beqz
                            let reg = seqz.src().reg().with_context(|| context!())?;
                            let label = beq.label().clone();
                            new_insts[*seqz_idx] = None;
                            new_insts[beq_idx] = Some(BneInst::new(reg, REG_ZERO, label).into());
                        };
                    }
                    Inst::Xor(xor) => {
                        let xor_idx = icmp_def_idx;
                        // snez 的 src 只能是 reg
                        let lhs = xor.lhs().reg().with_context(|| context!())?;
                        let label = beq.label().clone();
                        match xor.rhs() {
                            Operand::Reg(rhs) => {
                                new_insts[*xor_idx] = None;
                                let rhs = *rhs;
                                new_insts[beq_idx] = Some(BeqInst::new(lhs, rhs, label).into());
                            }
                            Operand::Imm(imm) => {
                                // 重复利用这个 reg 和 槽
                                let rhs = xor.dst().reg().with_context(|| context!())?;
                                new_insts[*xor_idx] =
                                    Some(LiInst::new(rhs.into(), (*imm).into()).into());
                                new_insts[beq_idx] = Some(BeqInst::new(lhs, rhs, label).into());
                            }
                            _ => unimplemented!(),
                        }
                    }
                    Inst::Slt(slt) => {
                        let slt_idx = icmp_def_idx;
                        // snez 的 src 只能是 reg
                        let lhs = slt.lhs().reg().with_context(|| context!())?;
                        let label = beq.label().clone();
                        match slt.rhs() {
                            Operand::Reg(rhs) => {
                                new_insts[*slt_idx] = None;
                                let rhs = *rhs;
                                new_insts[beq_idx] = Some(BgeInst::new(lhs, rhs, label).into());
                            }
                            Operand::Imm(imm) => {
                                // 重复利用这个 reg 和 槽
                                let rhs = slt.dst().reg().with_context(|| context!())?;
                                new_insts[*slt_idx] =
                                    Some(LiInst::new(rhs.into(), (*imm).into()).into());
                                new_insts[beq_idx] = Some(BgeInst::new(lhs, rhs, label).into());
                            }
                            _ => unimplemented!(),
                        }
                    }
                    Inst::Sltu(_) => todo!(),
                    Inst::Sgtu(_) => todo!(),
                    Inst::Seqz(_) => todo!(),
                    Inst::Snez(_) => todo!(),
                    Inst::Feqs(_) => todo!(),
                    Inst::Fles(_) => todo!(),
                    Inst::Flts(_) => todo!(),
                    _ => unimplemented!(),
                };
            }
        }

        *block.insts_mut() = new_insts.into_iter().flatten().collect();
        Ok(())
    }

    /// this function should be call in abstract asmbly stage
    pub fn combine_for_gep(block: &mut Block) -> Result<()> {
        // 主要处理指令:add,sll,sw,lw
        let mut reg_imms: HashMap<Reg, Imm> = HashMap::new();

        let get_imm = |op: &Operand, reg_vals: &HashMap<Reg, Imm>| -> Option<Imm> {
            if let Operand::Imm(imm) = op {
                Some(*imm)
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
                Inst::Div(div) => {
                    if let Some(rhs) = get_imm(div.rhs(), &reg_imms) {
                        *div.rhs_mut() = Operand::Imm(rhs);
                    }
                }
                Inst::Sub(sub) => {
                    if let Some(rhs) = get_imm(sub.rhs(), &reg_imms) {
                        *sub.rhs_mut() = Operand::Imm(rhs);
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
            if let Some(val) = dst_val {
                assert_eq!(inst.defs().len(), 1);
                let dst = inst.defs().first().cloned().unwrap();
                reg_imms.insert(*dst, val);
                *inst = LiInst::new(dst.into(), val.into()).into();
            } else {
                for reg in inst.defs() {
                    reg_imms.remove(reg);
                }
            }
        }
        Ok(())
    }

    pub fn combine_for_gep2(block: &mut Block) -> Result<()> {
        #[derive(Debug, Clone, Copy)]
        enum Loc {
            Unknown,
            BaseOff(Reg, Imm),
            Off(Imm),
        }

        impl std::ops::Add for Loc {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                match (self, rhs) {
                    (Loc::Unknown, _) | (_, Loc::Unknown) => Loc::Unknown,
                    (Loc::BaseOff(base1, off1), Loc::BaseOff(base2, off2)) => {
                        if base1 == base2 {
                            Loc::BaseOff(base1, off1 + off2)
                        } else {
                            Loc::Unknown
                        }
                    }
                    (Loc::BaseOff(base, off), Loc::Off(off2)) => Loc::BaseOff(base, off + off2),
                    (Loc::Off(off), Loc::BaseOff(base, off2)) => Loc::BaseOff(base, off + off2),
                    (Loc::Off(off), Loc::Off(off2)) => Loc::Off(off + off2),
                }
            }
        }

        impl std::ops::Shl<Loc> for Loc {
            type Output = Self;
            fn shl(self, rhs: Loc) -> Self {
                match (self, rhs) {
                    (Loc::Unknown, _) | (_, Loc::Unknown) => Loc::Unknown,
                    (Loc::BaseOff(_, _), Loc::Off(off2)) => {
                        if off2 == 0.into() {
                            self
                        } else {
                            Loc::Unknown
                        }
                    }
                    (Loc::Off(off), Loc::Off(off2)) => {
                        if let Ok(off2) = off2.try_into() {
                            Loc::Off(off << off2)
                        } else {
                            Loc::Unknown
                        }
                    }
                    _ => Loc::Unknown,
                }
            }
        }

        let mut reg_vals: HashMap<Reg, Loc> = HashMap::new();
        let get_val = |op: &Operand, reg_vals: &HashMap<Reg, Loc>| -> Option<Loc> {
            if let Operand::Reg(reg) = op {
                reg_vals.get(reg).cloned()
            } else if let Operand::Imm(imm) = op {
                Some(Loc::Off(*imm))
            } else {
                None
            }
        };

        for inst in block.insts_mut() {
            // 如果类型是lw/sw/ld/sd, 且地址已经被计算出来,则可以进行优化
            macro_rules! opt_ls {
                ($ls_ty:ident,$inst:expr,$get_val:ident,$regs_val:ident) => {
                    if let Some(addr) = $get_val(&$inst.base().into(), &$regs_val) {
                        let off = $inst.offset();
                        let new_addr = addr + Loc::Off(*off);
                        if let Loc::BaseOff(base, off) = new_addr {
                            *$inst = $ls_ty::new(*$inst.dst(), off, base);
                        }
                    }
                };
            }
            match inst {
                Inst::Lw(lw) => {
                    opt_ls!(LwInst, lw, get_val, reg_vals);
                }
                Inst::Sw(sw) => {
                    opt_ls!(SwInst, sw, get_val, reg_vals);
                }
                Inst::Ld(ld) => {
                    opt_ls!(LdInst, ld, get_val, reg_vals);
                }
                Inst::Sd(sd) => {
                    opt_ls!(SdInst, sd, get_val, reg_vals);
                }
                _ => {}
            }

            let dst_val: Option<Loc> = match inst {
                Inst::LocalAddr(local_addr) => Loc::BaseOff(*local_addr.dst(), 0.into()).into(),
                Inst::Add(add) => {
                    if let (Some(lhs), Some(rhs)) =
                        (get_val(add.lhs(), &reg_vals), get_val(add.rhs(), &reg_vals))
                    {
                        (lhs + rhs).into()
                    } else {
                        None
                    }
                }
                Inst::Sll(sll) => {
                    if let (Some(lhs), Some(rhs)) =
                        (get_val(sll.lhs(), &reg_vals), get_val(sll.rhs(), &reg_vals))
                    {
                        (lhs << rhs).into()
                    } else {
                        None
                    }
                }
                Inst::Li(li) => Some(Loc::Off(li.src().try_into()?)),
                _ => None,
            };

            //刷新值
            for r in inst.defs() {
                if let Some(dst_val) = dst_val {
                    reg_vals.insert(*r, dst_val);
                } else {
                    reg_vals.remove(r);
                }
            }
        }
        Ok(())
    }

    pub fn rm_useless_def_reg(bb: &mut Block, live_out: &FxHashSet<Reg>) -> Result<()> {
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
    fn test_combine_for_gep2() {
        // load_addr x37,[0-40]
        // addiw x38,x37,16
        // lw x39,0(x38)

        // such should be optimized to
        // load_addr x37,[0-40]
        // lw x39,14(x37)
        let mut bb = Block::new("test".to_string());
        let mut ssa = StackAllocator::new();
        let x37 = Reg::new(37, true);
        let x38 = Reg::new(38, true);
        let x39 = Reg::new(39, true);
        bb.push_inst(LocalAddr::new(x37, ssa.alloc(40)).into());
        bb.push_inst(AddInst::new(x38.into(), x37.into(), 16.into()).into());
        bb.push_inst(LwInst::new(x39, 0.into(), x38).into());
        let asm_before = bb.gen_asm();
        Block::combine_for_gep2(&mut bb).unwrap();
        let asm_after = bb.gen_asm();
        assert_snapshot!(diff(&asm_before, &asm_after),@r###"
        test:
        load_addr x37,[0-40]
        addiw x38,x37,16
        [-] lw x39,0(x38)
        [+] lw x39,16(x37)
        "###);

        Block::rm_useless_def_reg(&mut bb, &vec![x39].into_iter().collect()).unwrap();
        let asm_after2 = bb.gen_asm();
        assert_snapshot!(diff(&asm_after, &asm_after2),@r###"
        test:
        load_addr x37,[0-40]
        [-] addiw x38,x37,16
        lw x39,16(x37)
        "###);
    }

    #[test]
    fn test_remove_useless_def_reg() {
        let mut bb = Block::new("t".to_string());
        *bb.insts_mut() = vec![
            LiInst::new(REG_A0.into(), 0.into()).into(),
            LiInst::new(REG_S1.into(), 0.into()).into(),
        ];
        let live_out: FxHashSet<Reg> = vec![REG_S1].into_iter().collect();
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
        let live_out: FxHashSet<Reg> = vec![REG_ZERO].into_iter().collect();
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
