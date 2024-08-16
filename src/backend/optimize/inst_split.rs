use crate::utils::vec_util;

use super::*;

pub fn handle_inst_split(func: &mut Func) -> Result<()> {
    Func::mul_div_opt(func)?;
    // Func::split_li(func)?;
    Ok(())
}

impl Func {
    /// 处理乘法和除法的优化,部分乘法和除法可以 优化成移位
    fn mul_div_opt(func: &mut Func) -> Result<()> {
        let mut r_g = func
            .reg_gener_mut()
            .take()
            .ok_or(anyhow!("msg: reg_gener not found"))
            .with_context(|| context!())?;

        func.iter_bbs_mut()
            .try_for_each(|bb| Block::mul_div_opt(bb, &mut r_g))?;

        func.reg_gener_mut().replace(r_g);
        Ok(())
    }

    /// handle li , li
    fn split_li(func: &mut Func) -> Result<()> {
        for block in func.iter_bbs_mut() {
            let mut new_insts = Vec::new();
            for i in block.insts_mut() {
                if let Inst::Li(li) = i {
                    let imm = li.src().imm().ok_or_else(|| anyhow!(""))?;
                    if imm.in_limit_12() {
                        new_insts.push(li.clone().into());
                    } else {
                        // FIXME: 这里需要拆分成多条指令
                        todo!();
                    }
                } else {
                    new_insts.push(i.clone());
                }
            }
            *block.insts_mut() = new_insts;
        }

        Ok(())
    }
}

impl Block {
    fn mul_div_opt(bb: &mut Block, r_g: &mut RegGenerator) -> Result<()> {
        let mut new_insts: Vec<Inst> = Vec::new();
        for inst in bb.insts_mut() {
            match inst {
                Inst::Mul(mul) => Inst::mul_opt(mul, r_g, &mut new_insts),
                Inst::Div(div) => Inst::div_opt(div, r_g, &mut new_insts),
                Inst::Rem(rem) => Inst::rem_opt(rem, r_g, &mut new_insts),
                _ => {
                    new_insts.push(inst.clone()); // 这里就是啥也不干
                }
            }
        }
        *bb.insts_mut() = new_insts;
        Ok(())
    }
}

impl Inst {
    /// 如果对 2^n 取余, 可以优化成与操作
    fn rem_opt(rem: &mut RemInst, r_g: &mut RegGenerator, new_insts: &mut Vec<Inst>) {
        if let Operand::Imm(imm) = rem.rhs() {
            let num = **imm;
            if (num & (num - 1) == 0) && (num > 0) {
                let power = num.trailing_zeros();
                let mask = (1 << power) - 1;
                let andi = AndInst::new(rem.dst().clone(), rem.lhs().clone(), (mask as i64).into());
                new_insts.push(andi.into());
            } else {
                let mid = r_g.gen_virtual_usual_reg();
                let li = LiInst::new(mid.into(), imm.into());
                *rem.rhs_mut() = Operand::Reg(mid);
                new_insts.push(li.into());
                new_insts.push(rem.clone().into());
            }
        } else {
            new_insts.push(rem.clone().into());
        }
    }

    /// 除法只有在除数是 2 的幂次方时才能优化
    fn div_opt(div: &mut DivInst, r_g: &mut RegGenerator, new_insts: &mut Vec<Inst>) {
        if let Operand::Imm(imm) = div.rhs() {
            let num = **imm;
            if (num & (num - 1) == 0) && (num > 0) {
                let power = num.trailing_zeros();
                let srai =
                    SraInst::new(div.dst().clone(), div.lhs().clone(), (power as i64).into());
                new_insts.push(srai.into());
            } else {
                let mid = r_g.gen_virtual_usual_reg();
                let li = LiInst::new(mid.into(), imm.into());
                *div.rhs_mut() = Operand::Reg(mid);
                new_insts.push(li.into());
                new_insts.push(div.clone().into());
            }
        } else {
            new_insts.push(div.clone().into());
        }
    }

    fn mul_opt(mul: &mut MulInst, r_g: &mut RegGenerator, new_insts: &mut Vec<Inst>) {
        /// (1 << m) - (1 << n)
        fn _is_sub_pattern(num: i64) -> Option<(u32, u32)> {
            /// 最小的, 大于 num 的, 二次幂
            fn _next_power_of_two_log(num: i64) -> u32 {
                if num <= 0 {
                    return 1; // 对于非正数，返回1
                }
                // 计算最近的大于等于num的二次幂
                let mut v = num - 1;
                v |= v >> 1;
                v |= v >> 2;
                v |= v >> 4;
                v |= v >> 8;
                v |= v >> 16;
                v |= v >> 32;
                (v + 1).trailing_zeros()
            }
            /* ---------- 函数本体 ---------- */
            let m = _next_power_of_two_log(num);
            for n in 0..m {
                if (1 << m) - (1 << n) == num {
                    return Some((m, n));
                }
            }
            None
        }

        if let Operand::Imm(imm) = mul.rhs() {
            let num = **imm;
            let ones: Vec<u32> = (0..64).filter(|&i| (num >> i) & 1 == 1).collect();
            if let Some((m, n)) = _is_sub_pattern(num) {
                let lhs = mul.lhs();
                let sll_m = r_g.gen_virtual_usual_reg();
                let sll = SllInst::new(sll_m.into(), lhs.clone(), (m as i64).into());
                new_insts.push(sll.into());
                let sll_n = r_g.gen_virtual_usual_reg();
                let sll = SllInst::new(sll_n.into(), lhs.clone(), (n as i64).into());
                new_insts.push(sll.into());
                let dst = mul.dst();
                let sub = SubInst::new(dst.clone(), sll_m.into(), sll_n.into());
                new_insts.push(sub.into());
            } else if num == 0 {
                let dst = mul.dst();
                let addi = AddInst::new(dst.clone(), REG_ZERO.into(), REG_ZERO.into());
                new_insts.push(addi.into());
            } else if ones.len() <= 3 {
                // (((1 + x) + y) + z)
                let lhs = mul.lhs();
                let first = &ones[0]; // 不会出现 ones.len() == 0 的情况, 因为 ones.len() ==0 则 num == 0
                let dst = mul.dst();
                let sll = SllInst::new(dst.clone(), lhs.clone(), (*first as i64).into());
                new_insts.push(sll.into());
                let rest = &ones[1..];
                for r in rest {
                    let _sll_r = r_g.gen_virtual_usual_reg();
                    let sll = SllInst::new(_sll_r.into(), lhs.clone(), (*r as i64).into());
                    new_insts.push(sll.into());
                    let add = AddInst::new(dst.clone(), dst.clone(), _sll_r.into());
                    new_insts.push(add.into());
                }
            } else {
                let mid = r_g.gen_virtual_usual_reg();
                let li = LiInst::new(mid.into(), imm.into());
                new_insts.push(li.into());
                *mul.rhs_mut() = Operand::Reg(mid);
                new_insts.push(mul.clone().into());
            }
        } else {
            new_insts.push(mul.clone().into());
        }
    }
}
