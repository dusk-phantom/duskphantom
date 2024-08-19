pub use super::*;
use anyhow::Ok;
use core::ffi;
use rustc_hash::{FxHashMap, FxHashSet};

pub fn try_perfect_alloc(
    reg_graph: &FxHashMap<Reg, FxHashSet<Reg>>,
    def_then_def: &FxHashMap<Reg, FxHashSet<Reg>>,
    could_merge: &[((Reg, Reg), usize)],
) -> Result<FxHashMap<Reg, Reg>> {
    let u_regs = free_uregs_with_tmp();
    let f_regs = free_fregs_with_tmp();
    let mut reg_graph = reg_graph.clone();

    merge_regs(&mut reg_graph, could_merge, u_regs.len(), f_regs.len())
        .with_context(|| context!())
        .unwrap();

    assign_extra_edge(&mut reg_graph, u_regs.len(), f_regs.len(), def_then_def);
    let (simplified_graph, mut later_to_color) = simplify_graph(&reg_graph, u_regs, f_regs);
    if simplified_graph
        .iter()
        .filter(|(r, _)| r.is_virtual())
        .count()
        != 0
    {
        return Err(anyhow!(""));
    }
    let mut colors: FxHashMap<Reg, Reg> = FxHashMap::default();

    while let Some(r) = later_to_color.pop_back() {
        let interred_colors = physical_inters(&reg_graph, Some(&colors), &r);
        if let Some(color) = select_free_color(u_regs, f_regs, &r, &interred_colors) {
            colors.insert(r, color);
        } else {
            return Err(anyhow!(""));
        }
    }

    Ok(colors)
}

pub fn apply_colors(func: &mut Func, colors: FxHashMap<Reg, Reg>) {
    for block in func.iter_bbs_mut() {
        for inst in block.insts_mut() {
            let uses: Vec<Reg> = inst.uses().into_iter().cloned().collect();
            let defs: Vec<Reg> = inst.defs().into_iter().cloned().collect();
            for r in uses.iter().filter(|r| r.is_virtual()) {
                if let Some(color) = colors.get(r) {
                    inst.replace_use(*r, *color);
                }
            }
            for r in defs.into_iter().filter(|r| r.is_virtual()) {
                if let Some(color) = colors.get(&r) {
                    inst.replace_def(r, *color);
                }
            }
        }
    }
}

/// FIXME: some bug exists
/// 使用t0-t2来处理spill的虚拟寄存器
pub fn apply_spills(func: &mut Func, spills: FxHashSet<Reg>) {
    if spills.is_empty() {
        return;
    }
    phisicalize::phisicalize_reg(func);
}

/// FIXME: now have some bug, need more precise analysis for reg lives
/// 延迟t0-t2的释放的方式来处理spill的虚拟寄存器,
/// 也就是在使用到spill的虚拟寄存器时,选择t0-t2中一个将其物理化,在使用完后,直到被下一个spill虚拟寄存器使用前,才释放占有的物理寄存器
pub fn apply_spills2(func: &mut Func, spills: FxHashSet<Reg>) -> Result<()> {
    if spills.is_empty() {
        return Ok(());
    }
    let reg_lives = Func::reg_lives(func)?;

    let mut ssa = func
        .stack_allocator_mut()
        .take()
        .ok_or(anyhow!("stack allocator is none"))?;
    let mut v_ss = FxHashMap::default();
    let mut get_ss_for_spill = |r: &Reg| -> Result<StackSlot> {
        if let Some(ss) = v_ss.get(r) {
            return Ok(*ss);
        }
        let ss = ssa.alloc(8);
        v_ss.insert(*r, ss);
        Ok(ss)
    };

    // key: virtual reg, value: the physical reg that is used to store the value of the virtual reg
    let mut owner: FxHashMap<Reg, Reg> = FxHashMap::default();
    let mut owned: FxHashSet<Reg> = FxHashSet::default();
    let i_tmps = tmp_u_regs();
    let f_tmps = tmp_f_regs();
    for block in func.iter_bbs_mut() {
        let mut new_insts = vec![];
        for inst in block.insts_mut() {
            let uses: Vec<Reg> = inst.uses().into_iter().cloned().collect();
            let defs: Vec<Reg> = inst.defs().into_iter().cloned().collect();
            let mut used: FxHashSet<Reg> = FxHashSet::default();
            let mut to_add_before = vec![];
            let mut to_add_after = vec![];
            for r in uses.iter().filter(|r| spills.contains(r)) {
                if let Some(phy) = owner.get(r) {
                    inst.replace_use(*r, *phy);
                    used.insert(*phy);
                } else {
                    let phy = if r.is_usual() {
                        i_tmps.iter().find(|r| !owned.contains(r))
                    } else {
                        f_tmps.iter().find(|r| !owned.contains(r))
                    };
                    if let Some(phy) = phy {
                        used.insert(*phy);
                        // load the value of the virtual reg to the physical reg
                        let ss = get_ss_for_spill(r)?;
                        let load = LoadInst::new(*phy, ss);
                        to_add_before.push(load.into());
                        owner.insert(*r, *phy);
                        owned.insert(*phy);
                        inst.replace_use(*r, *phy);
                    } else {
                        // release one of the tmps
                        let phy = if r.is_usual() {
                            i_tmps.iter().find(|r| !used.contains(r))
                        } else {
                            f_tmps.iter().find(|r| !used.contains(r))
                        }
                        .unwrap();
                        if let Some((k, _)) = owner.iter().find(|(_, v)| *v == phy) {
                            let k = *k;
                            owner.remove(&k);
                            owned.remove(phy);
                            // write back old value to stack_slot
                            let ss = get_ss_for_spill(&k)?;
                            let store = StoreInst::new(ss, *phy);
                            to_add_before.push(store.into());
                        }

                        used.insert(*phy);

                        // load the value of the virtual reg to the physical reg
                        let ss = get_ss_for_spill(r)?;
                        let load = LoadInst::new(*phy, ss);
                        to_add_before.push(load.into());
                        owner.insert(*r, *phy);
                        owned.insert(*phy);
                    }
                }
            }

            for r in defs.into_iter().filter(|r| spills.contains(r)) {
                if let Some(phy) = owner.get(&r) {
                    inst.replace_def(r, *phy);
                } else {
                    let phy = if r.is_usual() {
                        i_tmps.iter().find(|r| !owned.contains(r))
                    } else {
                        f_tmps.iter().find(|r| !owned.contains(r))
                    };
                    if let Some(phy) = phy {
                        owner.insert(r, *phy);
                        owned.insert(*phy);
                        inst.replace_def(r, *phy);
                    } else {
                        // release one of the tmps
                        let phy = if r.is_usual() {
                            i_tmps.iter().find(|r| !used.contains(r))
                        } else {
                            f_tmps.iter().find(|r| !used.contains(r))
                        }
                        .unwrap();
                        if let Some((k, _)) = owner.iter().find(|(_, v)| *v == phy) {
                            let k = *k;
                            owner.remove(&k);
                            owned.remove(phy);
                            // write back old value to stack_slot
                            let ss = get_ss_for_spill(&k)?;
                            let store = StoreInst::new(ss, *phy);
                            to_add_before.push(store.into());
                        }
                        owner.insert(r, *phy);
                        owned.insert(*phy);
                    }
                }
            }

            new_insts.extend(to_add_before);
            new_insts.push(inst.clone());
            new_insts.extend(to_add_after);
        }

        *block.insts_mut() = new_insts;

        let mut insert_before_term = vec![];
        for (owner, phy) in owner.iter() {
            if reg_lives.live_outs(block).contains(owner) {
                let ss = get_ss_for_spill(phy)?;
                let store = StoreInst::new(ss, *owner);
                insert_before_term.push(store.into());
            }
        }
        for inst in insert_before_term {
            block.insert_before_term(inst);
        }
    }

    func.stack_allocator_mut().replace(ssa);
    Ok(())
}

/// register allocation, return the mapping from virtual reg to physical reg, and the set of regs that need to be spilled
/// # Arguments
/// * `graph` - the interference graph
/// # Returns
/// `Result<(colors: FxHashMap<Reg, Reg>, to_spill: FxHashSet<Reg>)>`
/// - colors: the mapping from virtual reg to physical reg
/// - to_spill: the set of regs that need to be spilled
///
pub fn reg_alloc(
    graph: &FxHashMap<Reg, FxHashSet<Reg>>,
    i_colors: &[Reg],
    f_colors: &[Reg],
    costs: Option<&FxHashMap<Reg, usize>>,
) -> Result<(FxHashMap<Reg, Reg>, FxHashSet<Reg>)> {
    let (graph_to_simplify, mut later_to_color) = simplify_graph(graph, i_colors, f_colors);

    let mut colors: FxHashMap<Reg, Reg> = FxHashMap::default();
    let mut to_spill: FxHashSet<Reg> = FxHashSet::default();

    // try to color the rest of the graph
    let mut first_to_color: Vec<(Reg, usize)> = graph_to_simplify
        .into_iter()
        .filter(|(k, _)| k.is_virtual())
        .map(|(k, v)| (k, v.len()))
        .collect();
    first_to_color.sort_by_key(|(_, v)| *v);
    for (k, _) in first_to_color {
        later_to_color.push_back(k);
    }

    while let Some(r) = later_to_color.pop_back() {
        let mut used_colors: FxHashSet<Reg> = FxHashSet::default();
        let inter = graph.get(&r).expect("");
        for v in inter {
            if v.is_physical() {
                used_colors.insert(*v);
            }
            if let Some(c) = colors.get(v) {
                used_colors.insert(*c);
            }
        }
        // find the first color that is not used
        let color = if r.is_usual() {
            i_colors.iter().find(|c| !used_colors.contains(c))
        } else {
            f_colors.iter().find(|c| !used_colors.contains(c))
        };
        if let Some(color) = color {
            colors.insert(r, *color);
        } else {
            to_spill.insert(r);
        }
    }

    Ok((colors, to_spill))
}

/// return simplified graph and ordered later to color nodes
#[inline]
pub fn simplify_graph(
    graph: &FxHashMap<Reg, FxHashSet<Reg>>,
    i_colors: &[Reg],
    f_colors: &[Reg],
) -> (FxHashMap<Reg, FxHashSet<Reg>>, VecDeque<Reg>) {
    let mut graph_to_simplify = graph.clone();

    let mut later_to_color: VecDeque<Reg> = VecDeque::new();

    // simpilify the graph
    // if a node has less than K neighbors, remove it from the graph, and add it to the later_to_color
    loop {
        let mut to_remove = vec![];
        for (k, v) in graph_to_simplify.iter() {
            if k.is_physical() {
                continue;
            }
            let num_inter = v.iter().filter(|v| v.is_usual() == k.is_usual()).count();
            if k.is_float() {
                if num_inter < f_colors.len() {
                    to_remove.push(*k);
                    later_to_color.push_back(*k);
                }
            } else if k.is_usual() {
                if num_inter < i_colors.len() {
                    to_remove.push(*k);
                    later_to_color.push_back(*k);
                }
            } else {
                unreachable!("a reg can only be usual or float");
            }
        }

        if to_remove.is_empty() {
            break;
        }
        for r in to_remove {
            remove_node(&mut graph_to_simplify, r);
        }
    }

    (graph_to_simplify, later_to_color)
}

// 给图加上附加边,在不超过最佳范围的情况
// 要求: 输入的图应该是个无向图,如果不是,执行结果可能不符合预期
pub fn assign_extra_edge(
    graph: &mut FxHashMap<Reg, FxHashSet<Reg>>,
    num_free_iregs: usize,
    num_free_fregs: usize,
    mut extra_edges: &FxHashMap<Reg, FxHashSet<Reg>>,
) {
    fn num_inter(g: &FxHashMap<Reg, FxHashSet<Reg>>, r: &Reg) -> usize {
        g.get(r).map(|nbs| nbs.len()).unwrap_or(0)
    }
    fn inter(g: &FxHashMap<Reg, FxHashSet<Reg>>, r1: &Reg, r2: &Reg) -> bool {
        g.get(r1).map(|nbs| nbs.contains(r2)).unwrap_or(false)
    }

    for (r1, r2) in extra_edges {
        for r2 in r2 {
            // case1: 相同的寄存器不能加边
            // case2: 已经存在冲突关系的,不需要加边
            // case3: 类型不同的寄存器,不需要加边
            // case4: 两个寄存器都是物理寄存器,不需要加边
            if r1 == r2
                || inter(graph, r1, r2)
                || r1.is_usual() != r2.is_usual()
                || (r1.is_physical() && r2.is_physical())
            {
                continue;
            }
            let num_inter1 = num_inter(graph, r1);
            let num_inter2 = num_inter(graph, r2);
            let num_max_free = if r1.is_usual() {
                num_free_iregs
            } else {
                num_free_fregs
            };
            if num_inter1 + 1 < num_max_free && num_inter2 + 1 < num_max_free {
                graph.entry(*r1).or_default().insert(*r2);
                graph.entry(*r2).or_default().insert(*r1);
            }
        }
    }
}

//////////////////////////////////////////////////////
/// some helper functions
//////////////////////////////////////////////////////

/// generate the interference graph txt for the function
pub fn g2txt(g: &FxHashMap<Reg, FxHashSet<Reg>>) -> String {
    let mut s = String::new();
    s.push_str("{\n");
    for (k, v) in g {
        s.push('{');
        s.push_str(&format!("{} -> ", k.gen_asm()));
        let mut v = v.iter();
        if let Some(r) = v.next() {
            s.push_str(&r.gen_asm());
        }
        for r in v {
            s.push_str(&format!(",{}", r.gen_asm()));
        }
        s.push_str("},\n");
    }
    s.push('}');
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph::{udgraph, UdGraph};
    fn check_alloc(
        graph: &FxHashMap<Reg, FxHashSet<Reg>>,
        colors: &FxHashMap<Reg, Reg>,
        to_spill: &FxHashSet<Reg>,
    ) {
        for (k, v) in graph.iter() {
            if to_spill.contains(k) {
                continue;
            }
            let k_color = colors.get(k).unwrap();
            let mut inter_colors = FxHashSet::default();
            for r in v {
                if r.is_physical() {
                    inter_colors.insert(*r);
                    continue;
                }
                if to_spill.contains(r) {
                    continue;
                }
                inter_colors.insert(*colors.get(r).unwrap());
            }

            assert!(!inter_colors.contains(k_color));
        }
    }

    impl std::str::FromStr for Reg {
        type Err = anyhow::Error;
        fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
            for r in Reg::physical_regs() {
                if r.gen_asm() == s {
                    return Ok(*r);
                }
            }

            let (id, is_usual) = if let Some(id) = s.strip_prefix('x') {
                (id.parse::<u32>()?, true)
            } else if let Some(id) = s.strip_prefix('f') {
                (id.parse::<u32>()?, false)
            } else if let Some(id) = s.strip_prefix("vi") {
                (id.parse::<u32>()? + 32, true)
            } else if let Some(id) = s.strip_prefix("vf") {
                (id.parse::<u32>()? + 32, false)
            } else if let Some(id) = s.strip_prefix('v') {
                (id.parse::<u32>()? + 32, true)
            } else {
                return Err(anyhow!("invalid to parse reg from str {}", s));
            };

            Ok(Reg::new(id, is_usual))
        }
    }

    #[test]
    pub fn test_reg_alloc() {
        let mut graph = FxHashMap::default();
        let mut reg_gener = RegGenerator::new();
        let i_v1 = reg_gener.gen_virtual_reg(true);
        let i_v2 = reg_gener.gen_virtual_reg(true);
        let i_v3 = reg_gener.gen_virtual_reg(true);
        graph.insert(i_v1, FxHashSet::from_iter(vec![i_v2, i_v3]));
        graph.insert(i_v2, FxHashSet::from_iter(vec![i_v1, i_v3]));
        graph.insert(i_v3, FxHashSet::from_iter(vec![i_v1, i_v2]));
        let (colors, to_spill) =
            super::reg_alloc(&graph, free_uregs(), free_fregs(), None).unwrap();
        // dbg!(&colors);
        check_alloc(&graph, &colors, &to_spill);
    }

    #[test]
    fn t2() {
        let g: UdGraph<Reg> = udgraph!({v1->v2,v3}, {v2 -> v3},).unwrap();
        let g: HashMap<Reg, HashSet<Reg>> = g.into();
        let g: FxHashMap<Reg, HashSet<Reg>> = g
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect();
        let g: FxHashMap<Reg, FxHashSet<Reg>> = g
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect();
        // dbg!(&g);
        let (colors, spills) = reg_alloc(&g, &[REG_A0, REG_A1], &[], None).unwrap();
        // dbg!(&colors);
        // dbg!(&spills);
        assert!(spills.len() == 1);
        check_alloc(&g, &colors, &spills);

        let (colors, spills) = reg_alloc(&g, &[REG_A0], &[], None).unwrap();
        // dbg!(&colors);
        assert!(spills.len() == 2);
        check_alloc(&g, &colors, &spills);

        let (colors, spills) = reg_alloc(&g, &[REG_A0, REG_A1, REG_A2], &[], None).unwrap();
        // dbg!(&colors);
        assert!(spills.is_empty());
        check_alloc(&g, &colors, &spills);
    }
}
