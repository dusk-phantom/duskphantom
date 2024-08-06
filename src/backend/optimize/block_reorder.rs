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

#[allow(unused)]
/// FIXME: test needed
pub fn handle_single_jmp(func: &mut Func) -> Result<()> {
    let (ins, outs) = Func::in_out_bbs(func)?;
    let mut to_merge: HashMap<String, String> = HashMap::new();
    for bb in func.iter_bbs() {
        let outs_of_bb = outs.outs(bb);
        if let Some(out) = outs_of_bb.first() {
            if let Some(in_to_out) = ins.ins(out).first() {
                if in_to_out.label() == bb.label() {
                    to_merge.insert(bb.label().to_owned(), out.label().to_owned());
                }
            }
        }
    }

    to_merge.retain(|from, to| to != func.entry().label());

    while let Some((from, to)) = to_merge.iter().next() {
        func.merge_bb(from, to)?;
        let from = from.clone();
        let to = to.clone();
        to_merge.remove(&from);
        if let Some(to_to) = to_merge.remove(&to) {
            to_merge.insert(from, to_to);
        }
    }

    Ok(())
}
