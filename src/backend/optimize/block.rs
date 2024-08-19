use super::*;

/// 对于块Bi和块Bj,(i<j,i!=j-1)
/// 如果Bi的结尾的jmp指令跳转到Bj,则交换Bi+1和Bj的位置,能够省略jmp指令
/// 如果交换会造成需要插入新的jmp指令或者产生新的长跳转,则不交换
/// 否则,交换Bi+1和Bj的位置
/// 不断重复上述过程,直到不能再交换为止
#[allow(unused)]
pub fn handle_reorder(func: &mut Func) -> Result<()> {
    Ok(())
}

pub fn handle_block_simplify(func: &mut Func) -> Result<()> {
    while func.simplify_term()? && func.elim_empty_bb()? {}
    func.desimplify_term()?;
    Ok(())
}

/// FIXME: test needed
pub fn handle_single_jmp(func: &mut Func) -> Result<()> {
    let (ins, outs) = Func::in_out_bbs(func)?;

    let mut to_merge: HashMap<String, String> = HashMap::new();
    for bb in func.iter_bbs() {
        let outs_of_bb = outs.outs(bb);
        if outs_of_bb.len() != 1 {
            continue;
        }
        if let Some(out) = outs_of_bb.first() {
            let ins_of_out = ins.ins(out);
            if ins_of_out.len() != 1 {
                continue;
            }
            if let Some(in_to_out) = ins_of_out.first() {
                if in_to_out.label() == bb.label() {
                    to_merge.insert(bb.label().to_owned(), out.label().to_owned());
                }
            }
        }
    }

    to_merge.retain(|_, to| to != func.entry().label());

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

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::utils::diff::diff;

    use super::*;
    fn new_bb(label: &str) -> Block {
        Block::new(label.to_string())
    }
    fn j(bb: &mut Block, label: &str) {
        bb.push_inst(JmpInst::new(label.into()).into());
    }
    fn b(bb: &mut Block, label1: &str, label2: &str) {
        bb.push_inst(BeqInst::new(REG_A1, REG_ZERO, label1.into()).into());
        bb.push_inst(JmpInst::new(label2.into()).into());
    }
    #[test]
    fn t1() {
        let mut bb0 = new_bb("b0");
        bb0.push_inst(MvInst::new(REG_A0.into(), REG_A1.into()).into());
        j(&mut bb0, "b1");
        let mut bb1 = new_bb("b1");
        bb1.push_inst(MvInst::new(REG_A1.into(), REG_A2.into()).into());
        j(&mut bb1, "b2");
        let mut bb2 = new_bb("b2");
        bb2.push_inst(MvInst::new(REG_A2.into(), REG_A3.into()).into());
        j(&mut bb2, "b3");
        let mut bb3 = new_bb("b3");
        bb3.push_inst(Inst::Ret);
        let mut f = Func::new("".to_string(), vec![], bb0);
        f.push_bb(bb1);
        f.push_bb(bb2);
        f.push_bb(bb3);

        let f_before = f.gen_asm();

        handle_single_jmp(&mut f).unwrap();

        let f_after = f.gen_asm();

        assert_snapshot!(diff(&f_before,&f_after),@r###"
        .text
        .align	3
        .globl	
        .type	, @function
        :
        b0:
        mv a0,a1
        [-] j b1
        [-] b1:
        mv a1,a2
        [-] j b2
        [-] b2:
        mv a2,a3
        [-] j b3
        [-] b3:
        ret
        .size	, .-
        "###);
    }

    #[test]
    fn t2() {
        let mut bb0 = new_bb("b0");
        bb0.push_inst(MvInst::new(REG_A0.into(), REG_A1.into()).into());
        j(&mut bb0, "b1");
        let mut bb1 = new_bb("b1");
        bb1.push_inst(MvInst::new(REG_A1.into(), REG_A2.into()).into());
        b(&mut bb1, "b2", "b3");
        let mut bb2 = new_bb("b2");
        bb2.push_inst(MvInst::new(REG_A2.into(), REG_A3.into()).into());
        j(&mut bb2, "b3");
        let mut bb3 = new_bb("b3");
        bb3.push_inst(Inst::Ret);
        let mut f = Func::new("".to_string(), vec![], bb0);
        f.push_bb(bb1);
        f.push_bb(bb2);
        f.push_bb(bb3);

        let f_asm_before = f.gen_asm();

        handle_single_jmp(&mut f).unwrap();
        let f_asm_after = f.gen_asm();

        assert_snapshot!(diff(&f_asm_before,&f_asm_after),@r###"
        .text
        .align	3
        .globl	
        .type	, @function
        :
        b0:
        mv a0,a1
        [-] j b1
        [-] b1:
        mv a1,a2
        beq a1,zero,b2
        j b3
        b2:
        mv a2,a3
        j b3
        b3:
        ret
        .size	, .-
        "###);
    }

    #[test]
    fn t3() {}
}
