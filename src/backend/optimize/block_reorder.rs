use super::*;
/// 处理块的重新排序
pub fn handle_block_reordering(func: &mut Func) -> Result<()> {
    // FIXME
    Ok(())
}

pub fn handle_long_jump(func: &mut Func) -> Result<()> {
    // FIXME
    Ok(())
}

pub fn handle_single_jmp(func: &mut Func) -> Result<()> {
    let (ins, outs) = Func::in_out_bbs(func)?;
    let mut to_merge: Vec<(String, String)> = vec![];
    for bb in func.iter_bbs() {
        let outs_of_bb = outs.outs(bb);
        if outs_of_bb.len() == 1 {
            let out = outs_of_bb[0];
            if ins.ins(out).len() == 1 {
                to_merge.push((bb.label().to_owned(), out.label().to_owned()));
            }
        }
    }
    // FIXME
    Ok(())
}
