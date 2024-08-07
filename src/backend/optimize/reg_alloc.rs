use graph::UdGraph;

use crate::fprintln;

use super::*;

pub fn handle_reg_alloc(func: &mut Func) -> Result<()> {
    // count the interference graph
    let mut reg_graphs = Func::reg_interfere_graph(func)?;
    fprintln!("f_g.dot", "{}", func.bbs_graph_to_dot());
    fprintln!("log/reg_graphs.log", "{}", g2txt(&reg_graphs));
    let dot = UdGraph::<Reg>::from(reg_graphs.clone()).gen_dot("reg_graph", |r| r.gen_asm());
    fprintln!("graph.dot", "{}", dot);
    let (colors, spills) = reg_alloc(&reg_graphs, free_iregs(), free_fregs())?;

    apply_colors(func, colors);

    apply_spills(func, spills);

    Ok(())
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

/// 使用t0-t2来处理spill的虚拟寄存器
pub fn apply_spills(func: &mut Func, spills: HashSet<Reg>) {
    if spills.is_empty() {
        return;
    }
    phisicalize::phisicalize_reg(func);
}

/// 能够用于寄存器分配的寄存器,也就是除了特殊寄存器以外的寄存器, 这里的特殊寄存器包括: zero, ra, sp, gp, tp,s0,t0-t3 <br>
/// 其中t0-t3是临时寄存器,t0-t2用于处理spill的虚拟寄存器,t3用于计算内存操作指令off溢出时的地址 <br>
/// s0是栈帧指针,用于保存调用者保存的寄存器 <br>
/// ...
fn free_iregs() -> &'static [Reg; 22] {
    &[
        // usual registers
        REG_S1, REG_A0, REG_A1, REG_A2, REG_A3, REG_A4, REG_A5, REG_A6, REG_A7, REG_S2, REG_S3,
        REG_S4, REG_S5, REG_S6, REG_S7, REG_S8, REG_S9, REG_S10, REG_S11, REG_T4, REG_T5, REG_T6,
    ]
}

fn free_fregs() -> &'static [Reg; 29] {
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
    fn remove_node(g: &mut HashMap<Reg, HashSet<Reg>>, r: Reg) -> Result<()> {
        let nbs = g.remove(&r).unwrap_or_default();
        for nb in nbs {
            g.get_mut(&nb)
                .ok_or(anyhow!(
                    "neighbors of node {} must be in the graph",
                    r.gen_asm()
                ))?
                .remove(&r);
        }
        Ok(())
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
            remove_node(&mut graph_to_simplify, r)?;
        }
    }

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

    let ordered_to_color = later_to_color.into_iter().rev();
    for r in ordered_to_color {
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

//////////////////////////////////////////////////////
/// some helper functions
//////////////////////////////////////////////////////
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
            } else if let Some(id) = s.strip_prefix("v") {
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
        dbg!(&colors);
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
