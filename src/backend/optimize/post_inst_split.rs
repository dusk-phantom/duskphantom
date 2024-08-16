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
        let hi = (imm + 0x0080) >> 12; // 56
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
