use crate::utils::vec_util;

use super::*;

/// 处理乘法和除法的优化,部分乘法和除法可以 优化成移位
pub fn handle_mul_div_opt(func: &mut Func) -> Result<()> {
    /// FIXME: use optimize here
    phisicalize::handle_illegal_inst(func)?;
    Ok(())
}

/// handle li , li
pub fn handle_li(func: &mut Func) -> Result<()> {
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
