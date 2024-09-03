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

/// 对于块Bi和块Bj,(i<j,i!=j-1)
/// 如果Bi的结尾的jmp指令跳转到Bj,则交换Bi+1和Bj的位置,能够省略jmp指令
/// 如果交换会造成需要插入新的jmp指令或者产生新的长跳转,则不交换
/// 否则,交换Bi+1和Bj的位置
/// 不断重复上述过程,直到不能再交换为止
#[allow(unused)]
pub fn handle_reorder(func: &mut Func) -> Result<()> {
    let other_bbs = func.other_bbs();
    let new_order = PHCL::new(func)?.optimize_layout();
    let new_other_bbs = new_order
        .into_iter()
        .map(|id| other_bbs[id].clone())
        .collect::<Vec<_>>();
    *func.other_bbs_mut() = new_other_bbs;
    Ok(())
}

pub fn handle_block_simplify(func: &mut Func) -> Result<()> {
    while func.simplify_term()? && func.elim_empty_bb()? {}
    func.desimplify_term()?;
    Ok(())
}

/// a -> b 的权重
type Weight = f32;

/// 基本块在原数组中的下标, 这个下标在 other bbs 中是不变的
type BBIdx = usize;

/// grp 在 groups 中的下标, 注意, 这个下标是不断变化的
type GRPIdx = usize;

/// Pettis-Hansen code layout
pub struct PHCL {
    groups: Vec<Vec<BBIdx>>,
    edges: HashMap<(BBIdx /* from */, BBIdx /* to */), Weight /* weight */>,
}

impl PHCL {
    fn new(func: &Func) -> Result<Self> {
        // 初始化单元的 group
        let groups: Vec<Vec<BBIdx>> = func
            .other_bbs()
            .iter()
            .enumerate()
            .map(|(i, _)| vec![i])
            .collect();

        let label_idx_map: HashMap<String, BBIdx> = func
            .other_bbs()
            .iter()
            .enumerate()
            .map(|(i, bb)| (bb.label().to_string(), i))
            .collect();

        let mut edges = HashMap::new();
        for bb in func.iter_bbs().skip(1) {
            let to_bbs_label = Block::to_bbs(bb).with_context(|| context!())?;
            for to_bb_label in to_bbs_label {
                let to_bb = func.find_bb(&to_bb_label).with_context(|| context!())?;
                let weight: Weight = ((bb.depth + to_bb.depth) as Weight)
                    / ((bb.depth.abs_diff(to_bb.depth) as Weight) + 1.0);
                let to_bb_idx = label_idx_map
                    .get(&to_bb_label)
                    .with_context(|| context!())?;
                let bb_idx = label_idx_map.get(bb.label()).with_context(|| context!())?;
                edges.insert((*bb_idx, *to_bb_idx), weight);
            }
        }
        Ok(Self { groups, edges })
    }

    fn optimize_layout(&mut self) -> Vec<BBIdx> {
        // 1. 基本块分组
        while self.groups.len() > 1 {
            // 不断合并 grp, 合并到只有 1 个 grp
            if let Some((idx_a, idx_b)) = self.find_most_frequent_pair() {
                self.merge_groups(idx_a, idx_b);
            } else {
                break;
            }
        }

        // 2. 组间布局
        let mut layout: Vec<HashSet<BBIdx>> = Vec::new();
        while !self.groups.is_empty() {
            if let Some(best_group_idx) = self.select_best_group(&layout) {
                let best_grp: HashSet<BBIdx> =
                    self.groups.remove(best_group_idx).iter().cloned().collect();
                layout.push(best_grp);
            }
        }

        // 3. 生成最终布局
        let mut final_layout = Vec::new();
        for grp in layout {
            for blk in grp {
                final_layout.push(blk);
            }
        }

        final_layout
    }

    fn select_best_group(&self, layout: &[HashSet<BBIdx>]) -> Option<GRPIdx /* best_grp_idx */> {
        let mut max_weight: Weight = -1.0;
        let mut best_grp: Option<GRPIdx> = None;
        // groups 中的 grp , 不包含 layout 中的 grp
        for (idx, grp) in self.groups.iter().enumerate() {
            let mut total_weight: Weight = 0.0;
            for placed_grp in layout.iter() {
                for bb_a in grp {
                    for placed_bb in placed_grp {
                        if let Some(weight) = self.edges.get(&(*bb_a, *placed_bb)) {
                            total_weight += *weight;
                        }
                        if let Some(weight) = self.edges.get(&(*placed_bb, *bb_a)) {
                            total_weight += *weight;
                        }
                    }
                }
            }
            // 注意 total_weight 与 max_weight 的数值, 这可以保证: layout == NULL 的时候也可以使用
            if total_weight > max_weight {
                max_weight = total_weight;
                best_grp = Some(idx);
            }
        }
        best_grp
    }

    /// 找到权重最大的一对 group
    fn find_most_frequent_pair(&self) -> Option<(GRPIdx, GRPIdx)> {
        let mut max_weight: Weight = -1.0;
        let mut best_pair: Option<(GRPIdx, GRPIdx)> = None;
        for i /* 组下标 */ in 0..self.groups.len() {
            for j in i + 1..self.groups.len() {
                let mut total_weight : Weight = 0.0;
                for bb_a in &self.groups[i] {
                    for bb_b in &self.groups[j] {
                        if let Some( weight ) = self.edges.get( &(*bb_a, *bb_b) ) {
                            total_weight += *weight;
                        }
                        if let Some( weight ) = self.edges.get( &(*bb_b, *bb_a) ) {
                            total_weight += *weight;
                        }
                    }
                    if total_weight > max_weight {
                        max_weight = total_weight;
                        best_pair = Some((i, j));
                    }
                }
            }
        }
        best_pair
    }

    /// 合并两个 group
    fn merge_groups(&mut self, idx_a: GRPIdx, idx_b: GRPIdx) {
        let grp_b = self.groups.remove(idx_b);
        let grp_a = &mut self.groups[idx_a];
        for bb in grp_b {
            grp_a.push(bb);
        }
    }
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

    to_merge.retain(
        |_, to| to != func.entry().label(), /* 虽然我们不会有 -> entry */
    );

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

    use duskphantom_utils::diff::diff;

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

        let _f_before = format!("{:?}", &f);
        let f_asm_before = f.gen_asm();

        handle_single_jmp(&mut f).unwrap();
        let _f_after = format!("{:?}", &f);
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
