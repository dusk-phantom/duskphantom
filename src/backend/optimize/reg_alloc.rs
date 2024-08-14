use core::num;
use std::hash::Hash;

use graph::UdGraph;
use reg_set::RegSet;

use crate::fprintln;

use super::*;

pub fn handle_reg_alloc(func: &mut Func) -> Result<()> {
    if func.line() < 10000 {
        // count the interference graph
        let mut reg_graph = Func::reg_interfere_graph2(func)?;
        let waw = func.def_then_def2();
        assign_extra_edge2(&mut reg_graph, free_iregs().len(), free_fregs().len(), waw);
        if try_perfect_alloc2(func, &reg_graph).is_ok() {
            return Ok(());
        }
        let (colors, spills) = reg_alloc2(&reg_graph, free_iregs(), free_fregs())?;
        apply_colors(func, colors);
        apply_spills(func, spills);
    } else {
        let mut reg_graph = Func::reg_interfere_graph(func)?;
        let waw = func.def_then_def();
        assign_extra_edge(&mut reg_graph, free_iregs().len(), free_fregs().len(), waw);
        if try_perfect_alloc(func, &reg_graph).is_ok() {
            return Ok(());
        }
        let (colors, spills) = reg_alloc(&reg_graph, free_iregs(), free_fregs())?;
        apply_colors(func, colors);
        apply_spills(func, spills);
    }

    Ok(())
}

pub fn try_perfect_alloc(func: &mut Func, reg_graphs: &HashMap<Reg, HashSet<Reg>>) -> Result<()> {
    let mut i_regs: Vec<Reg> = free_iregs().to_vec();
    i_regs.extend(tmp_i_regs().iter().cloned());
    let mut f_regs: Vec<Reg> = free_fregs().to_vec();
    f_regs.extend(tmp_f_regs().iter().cloned());
    let (colors, spills) = reg_alloc(reg_graphs, &i_regs, &f_regs)?;
    if spills.is_empty() {
        apply_colors(func, colors);
        Ok(())
    } else {
        Err(anyhow!("spills is not empty,fail to allocate perfectly"))
    }
}

pub fn try_perfect_alloc2(func: &mut Func, reg_graphs: &HashMap<Reg, RegSet>) -> Result<()> {
    let mut i_regs: Vec<Reg> = free_iregs().to_vec();
    i_regs.extend(tmp_i_regs().iter().cloned());
    let mut f_regs: Vec<Reg> = free_fregs().to_vec();
    f_regs.extend(tmp_f_regs().iter().cloned());
    let (colors, spills) = reg_alloc2(reg_graphs, &i_regs, &f_regs)?;
    if spills.is_empty() {
        apply_colors(func, colors);
        Ok(())
    } else {
        Err(anyhow!("spills is not empty,fail to allocate perfectly"))
    }
}

pub fn apply_colors(func: &mut Func, colors: HashMap<Reg, Reg>) {
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
pub fn apply_spills(func: &mut Func, spills: HashSet<Reg>) {
    if spills.is_empty() {
        return;
    }
    phisicalize::phisicalize_reg(func);
}

/// FIXME: now have some bug, need more precise analysis for reg lives
/// 延迟t0-t2的释放的方式来处理spill的虚拟寄存器,
/// 也就是在使用到spill的虚拟寄存器时,选择t0-t2中一个将其物理化,在使用完后,直到被下一个spill虚拟寄存器使用前,才释放占有的物理寄存器
pub fn apply_spills2(func: &mut Func, spills: HashSet<Reg>) -> Result<()> {
    if spills.is_empty() {
        return Ok(());
    }
    let reg_lives = Func::reg_lives(func)?;

    let mut ssa = func
        .stack_allocator_mut()
        .take()
        .ok_or(anyhow!("stack allocator is none"))?;
    let mut v_ss = HashMap::new();
    let mut get_ss_for_spill = |r: &Reg| -> Result<StackSlot> {
        if let Some(ss) = v_ss.get(r) {
            return Ok(*ss);
        }
        let ss = ssa.alloc(8);
        v_ss.insert(*r, ss);
        Ok(ss)
    };

    // key: virtual reg, value: the physical reg that is used to store the value of the virtual reg
    let mut owner: HashMap<Reg, Reg> = HashMap::new();
    let mut owned: HashSet<Reg> = HashSet::new();
    let i_tmps = tmp_i_regs();
    let f_tmps = tmp_f_regs();
    for block in func.iter_bbs_mut() {
        let mut new_insts = vec![];
        for inst in block.insts_mut() {
            let uses: Vec<Reg> = inst.uses().into_iter().cloned().collect();
            let defs: Vec<Reg> = inst.defs().into_iter().cloned().collect();
            let mut used: HashSet<Reg> = HashSet::new();
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

/// 能够用于寄存器分配的寄存器,也就是除了特殊寄存器以外的寄存器, 这里的特殊寄存器包括: zero, ra, sp, gp, tp,s0,t0-t3 <br>
/// 其中t0-t3是临时寄存器,t0-t2用于处理spill的虚拟寄存器,t3用于计算内存操作指令off溢出时的地址 <br>
/// s0是栈帧指针,用于保存调用者保存的寄存器 <br>
/// ...
pub fn free_iregs() -> &'static [Reg; 22] {
    &[
        // usual registers
        REG_S1, REG_A0, REG_A1, REG_A2, REG_A3, REG_A4, REG_A5, REG_A6, REG_A7, REG_S2, REG_S3,
        REG_S4, REG_S5, REG_S6, REG_S7, REG_S8, REG_S9, REG_S10, REG_S11, REG_T4, REG_T5, REG_T6,
    ]
}

/// 除了ft0-ft2用于处理spill的虚拟寄存器,其他的都可以自由用于寄存器分配
pub fn free_fregs() -> &'static [Reg; 29] {
    // usual registers
    &[
        // float registers
        REG_FT3, REG_FT4, REG_FT5, REG_FT6, REG_FT7, REG_FS0, REG_FS1, REG_FA0, REG_FA1, REG_FA2,
        REG_FA3, REG_FA4, REG_FA5, REG_FA6, REG_FA7, REG_FS2, REG_FS3, REG_FS4, REG_FS5, REG_FS6,
        REG_FS7, REG_FS8, REG_FS9, REG_FS10, REG_FS11, REG_FT8, REG_FT9, REG_FT10, REG_FT11,
    ]
}

/// register allocation, return the mapping from virtual reg to physical reg, and the set of regs that need to be spilled
/// # Arguments
/// * `graph` - the interference graph
/// # Returns
/// `Result<(colors: HashMap<Reg, Reg>, to_spill: HashSet<Reg>)>`
/// - colors: the mapping from virtual reg to physical reg
/// - to_spill: the set of regs that need to be spilled
///
pub fn reg_alloc(
    graph: &HashMap<Reg, HashSet<Reg>>,
    i_colors: &[Reg],
    f_colors: &[Reg],
) -> Result<(HashMap<Reg, Reg>, HashSet<Reg>)> {
    let (graph_to_simplify, mut later_to_color) = simplify_graph(graph, i_colors, f_colors);

    let mut colors: HashMap<Reg, Reg> = HashMap::new();
    let mut to_spill: HashSet<Reg> = HashSet::new();

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
        let mut used_colors: HashSet<Reg> = HashSet::new();
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
    graph: &HashMap<Reg, HashSet<Reg>>,
    i_colors: &[Reg],
    f_colors: &[Reg],
) -> (HashMap<Reg, HashSet<Reg>>, VecDeque<Reg>) {
    fn remove_node(g: &mut HashMap<Reg, HashSet<Reg>>, r: Reg) {
        let nbs = g.remove(&r).unwrap_or_default();
        for nb in nbs {
            if let Some(nb_nbs) = g.get_mut(&nb) {
                nb_nbs.remove(&r);
            }
        }
    }

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

pub fn reg_alloc2(
    graph: &HashMap<Reg, RegSet>,
    i_colors: &[Reg],
    f_colors: &[Reg],
) -> Result<(HashMap<Reg, Reg>, HashSet<Reg>)> {
    let (graph_to_simplify, mut later_to_color) = simplify_graph2(graph, i_colors, f_colors);
    let mut colors: HashMap<Reg, Reg> = HashMap::new();
    let mut to_spill: HashSet<Reg> = HashSet::new();

    // try to color the rest of the graph
    let mut first_to_color: Vec<(Reg, usize)> = graph_to_simplify
        .into_iter()
        .filter(|(k, _)| k.is_virtual())
        .map(|(k, v)| {
            (
                k,
                if k.is_usual() {
                    v.num_regs_usual()
                } else {
                    v.num_regs_float()
                },
            )
        })
        .collect();
    first_to_color.sort_by_key(|(_, v)| *v);
    for (k, _) in first_to_color {
        later_to_color.push_back(k);
    }

    let ordered_to_color = later_to_color.into_iter().rev();

    for r in ordered_to_color {
        let mut used_colors: RegSet = RegSet::with_capacity(32);
        if let Some(inter) = graph.get(&r) {
            for v in inter.iter() {
                if v.is_physical() {
                    used_colors.insert(&v);
                } else if let Some(c) = colors.get(&v) {
                    used_colors.insert(c);
                }
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

pub fn simplify_graph2(
    graph: &HashMap<Reg, RegSet>,
    i_colors: &[Reg],
    f_colors: &[Reg],
) -> (HashMap<Reg, RegSet>, VecDeque<Reg>) {
    #[inline]
    fn remove_node(g: &mut HashMap<Reg, RegSet>, r: Reg) {
        let nbs = g.remove(&r).unwrap_or_default();
        for nb in nbs {
            if let Some(nb_nbs) = g.get_mut(&nb) {
                nb_nbs.remove(&r);
            }
        }
    }
    #[inline]
    fn num_inters(g: &HashMap<Reg, RegSet>, r: Reg) -> usize {
        g.get(&r)
            .map(|nbs| {
                if r.is_usual() {
                    nbs.num_regs_usual()
                } else {
                    nbs.num_regs_float()
                }
            })
            .unwrap_or(0)
    }

    let mut graph_to_simplify = graph.clone();
    let mut later_to_color: VecDeque<Reg> = VecDeque::new();
    // simpilify the graph
    // if a node has less than K neighbors, remove it from the graph, and add it to the later_to_color
    loop {
        let mut to_remove = vec![];
        for (k, _) in graph_to_simplify.iter() {
            if k.is_physical() {
                continue;
            }
            let num_inter = num_inters(&graph_to_simplify, *k);
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
    graph: &mut HashMap<Reg, HashSet<Reg>>,
    num_free_iregs: usize,
    num_free_fregs: usize,
    mut extra_edges: HashMap<Reg, HashSet<Reg>>,
) {
    fn num_inter(g: &HashMap<Reg, HashSet<Reg>>, r: &Reg) -> usize {
        g.get(r).map(|nbs| nbs.len()).unwrap_or(0)
    }
    fn inter(g: &HashMap<Reg, HashSet<Reg>>, r1: &Reg, r2: &Reg) -> bool {
        g.get(r1).map(|nbs| nbs.contains(r2)).unwrap_or(false)
    }

    for (r1, r2) in extra_edges {
        for r2 in r2.into_iter() {
            // case1: 相同的寄存器不能加边
            // case2: 已经存在冲突关系的,不需要加边
            // case3: 类型不同的寄存器,不需要加边
            // case4: 两个寄存器都是物理寄存器,不需要加边
            if r1 == r2
                || inter(graph, &r1, &r2)
                || r1.is_usual() != r2.is_usual()
                || (r1.is_physical() && r2.is_physical())
            {
                continue;
            }
            let num_inter1 = num_inter(graph, &r1);
            let num_inter2 = num_inter(graph, &r2);
            let num_max_free = if r1.is_usual() {
                num_free_iregs
            } else {
                num_free_fregs
            };
            if num_inter1 + 1 < num_max_free && num_inter2 + 1 < num_max_free {
                graph.entry(r1).or_default().insert(r2);
                graph.entry(r2).or_default().insert(r1);
            }
        }
    }
}

pub fn assign_extra_edge2(
    graph: &mut HashMap<Reg, RegSet>,
    num_free_fregs: usize,
    num_free_iregs: usize,
    mut extra_edges: HashMap<Reg, RegSet>,
) {
    fn num_inter(g: &HashMap<Reg, RegSet>, r: &Reg) -> usize {
        g.get(r)
            .map(|nbs| {
                if r.is_usual() {
                    nbs.num_regs_usual()
                } else {
                    nbs.num_regs_float()
                }
            })
            .unwrap_or(0)
    }
    fn inter(g: &HashMap<Reg, RegSet>, r1: &Reg, r2: &Reg) -> bool {
        g.get(r1).map(|nbs| nbs.contains(r2)).unwrap_or(false)
    }

    for (r1, r2) in extra_edges {
        for r2 in r2.iter() {
            // case1: 相同的寄存器不能加边
            // case2: 已经存在冲突关系的,不需要加边
            // case3: 类型不同的寄存器,不需要加边
            // case4: 两个寄存器都是物理寄存器,不需要加边
            if r1 == r2
                || inter(graph, &r1, &r2)
                || r1.is_usual() != r2.is_usual()
                || (r1.is_physical() && r2.is_physical())
            {
                continue;
            }
            let num_inter1 = num_inter(graph, &r1);
            let num_inter2 = num_inter(graph, &r2);
            let num_max_free = if r1.is_usual() {
                num_free_iregs
            } else {
                num_free_fregs
            };
            if num_inter1 + 1 < num_max_free && num_inter2 + 1 < num_max_free {
                graph.entry(r1).or_default().insert(&r2);
                graph.entry(r2).or_default().insert(&r1);
            }
        }
    }
}

//////////////////////////////////////////////////////
/// some helper functions
//////////////////////////////////////////////////////

/// generate the interference graph txt for the function
pub fn g2txt(g: &HashMap<Reg, HashSet<Reg>>) -> String {
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
        graph: &HashMap<Reg, HashSet<Reg>>,
        colors: &HashMap<Reg, Reg>,
        to_spill: &HashSet<Reg>,
    ) {
        for (k, v) in graph.iter() {
            if to_spill.contains(k) {
                continue;
            }
            let k_color = colors.get(k).unwrap();
            let mut inter_colors = HashSet::new();
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
        let mut graph = std::collections::HashMap::new();
        let mut reg_gener = RegGenerator::new();
        let i_v1 = reg_gener.gen_virtual_reg(true);
        let i_v2 = reg_gener.gen_virtual_reg(true);
        let i_v3 = reg_gener.gen_virtual_reg(true);
        graph.insert(i_v1, std::collections::HashSet::from_iter(vec![i_v2, i_v3]));
        graph.insert(i_v2, std::collections::HashSet::from_iter(vec![i_v1, i_v3]));
        graph.insert(i_v3, std::collections::HashSet::from_iter(vec![i_v1, i_v2]));
        let (colors, to_spill) = super::reg_alloc(&graph, free_iregs(), free_fregs()).unwrap();
        // dbg!(&colors);
        check_alloc(&graph, &colors, &to_spill);
    }
    #[test]
    fn t2() {
        let g: UdGraph<Reg> = udgraph!({v1->v2,v3}, {v2 -> v3},).unwrap();
        let g: HashMap<Reg, HashSet<Reg>> = g.into();
        // dbg!(&g);
        let (colors, spills) = reg_alloc(&g, &[REG_A0, REG_A1], &[]).unwrap();
        // dbg!(&colors);
        // dbg!(&spills);
        assert!(spills.len() == 1);
        check_alloc(&g, &colors, &spills);

        let (colors, spills) = reg_alloc(&g, &[REG_A0], &[]).unwrap();
        // dbg!(&colors);
        assert!(spills.len() == 2);
        check_alloc(&g, &colors, &spills);

        let (colors, spills) = reg_alloc(&g, &[REG_A0, REG_A1, REG_A2], &[]).unwrap();
        // dbg!(&colors);
        assert!(spills.is_empty());
        check_alloc(&g, &colors, &spills);
    }
}
