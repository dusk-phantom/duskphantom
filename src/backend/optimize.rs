use std::collections::{HashMap, HashSet, VecDeque};

use super::*;

#[allow(unused)]
pub fn optimize(program: &mut prog::Program) {
    phisicalize::phisicalize(program); // 直接物理化

    // FIXME: 优化
    // // 不调整块顺序的优化
    // for m in program.modules.iter_mut() {
    //     for f in m.funcs.iter_mut() {
    //         // inst combine? 匹配一些模式,将多条指令合并成一条
    //         // mul and div to shift
    //         // inst scheduling
    //         // register allocation
    //         // processing caller-save and callee-save
    //         // processing stack frame's opening and closing
    //         // block reordering
    //     }
    // }
}

// 能够用于寄存器分配的寄存器,也就是除了特殊寄存器以外的寄存器, 这里的特殊寄存器包括: zero, ra, sp, gp, tp,s0
fn available_iregs() -> &'static [Reg; 26] {
    &[
        // usual registers
        REG_T0, REG_T1, REG_T2, REG_S1, REG_A0, REG_A1, REG_A2, REG_A3, REG_A4, REG_A5, REG_A6,
        REG_A7, REG_S2, REG_S3, REG_S4, REG_S5, REG_S6, REG_S7, REG_S8, REG_S9, REG_S10, REG_S11,
        REG_T3, REG_T4, REG_T5, REG_T6,
    ]
}
fn available_fregs() -> &'static [Reg; 32] {
    // usual registers
    &[
        // float registers
        REG_FT0, REG_FT1, REG_FT2, REG_FT3, REG_FT4, REG_FT5, REG_FT6, REG_FT7, REG_FS0, REG_FS1,
        REG_FA0, REG_FA1, REG_FA2, REG_FA3, REG_FA4, REG_FA5, REG_FA6, REG_FA7, REG_FS2, REG_FS3,
        REG_FS4, REG_FS5, REG_FS6, REG_FS7, REG_FS8, REG_FS9, REG_FS10, REG_FS11, REG_FT8, REG_FT9,
        REG_FT10, REG_FT11,
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
pub fn reg_alloc(graph: &HashMap<Reg, HashSet<Reg>>) -> Result<(HashMap<Reg, Reg>, HashSet<Reg>)> {
    let mut graph_to_simplify = graph.clone();

    let mut later_to_color: VecDeque<Reg> = VecDeque::new();
    // simpilify the graph
    // if a node has less than K neighbors, remove it from the graph, and add it to the later_to_color
    let i_colors = available_iregs();
    let f_colors = available_fregs();
    loop {
        let mut to_remove = vec![];
        for (k, v) in graph_to_simplify.iter() {
            if k.is_physical() {
                continue;
            }
            let num_inter = v.iter().filter(|v| v.is_usual() == k.is_usual()).count();
            if k.is_float() {
                if num_inter >= f_colors.len() {
                    to_remove.push(*k);
                    later_to_color.push_back(*k);
                }
            } else if k.is_usual() {
                if num_inter >= i_colors.len() {
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
            let inter = graph_to_simplify.remove(&r).unwrap_or_default();
            for v in inter {
                graph_to_simplify
                    .get_mut(&v)
                    .expect(
                        "in a consistent reg inter graph ,v inter to v2,must means v2 inter to v",
                    )
                    .remove(&r);
            }
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

#[cfg(test)]
mod tests {
    use super::*;
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
        let (colors, to_spill) = super::reg_alloc(&graph).unwrap();
        dbg!(&colors);
        check_alloc(&graph, &colors, &to_spill);
    }
}
