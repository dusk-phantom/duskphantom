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

use std::default;

use anyhow::bail;

use super::*;

#[cfg(feature = "ilp_alloc")]
pub fn alloc(
    graph: &FxHashMap<Reg, FxHashSet<Reg>>,
    free_i_colors: &[Reg],
    free_f_colors: &[Reg],
    costs: Option<&FxHashMap<Reg, usize>>,
) -> Result<(FxHashMap<Reg, Reg>, FxHashSet<Reg>)> {
    let default_costs = FxHashMap::default();
    let costs = costs.unwrap_or(&default_costs);
    // 首先化简
    let (s_g, mut later_to_color) =
        graph_color::simplify_graph(graph, free_i_colors, free_f_colors);

    let mut colors = FxHashMap::default();
    let mut spills = FxHashSet::default();

    if s_g.len() < 100 {
        (colors, spills) = graph_color::alloc(&s_g, free_i_colors, free_f_colors, Some(costs))?;
    } else {
        // 分成两个图,分别是f图和i图
        let mut i_g: FxHashMap<Reg, FxHashSet<Reg>> = s_g.clone();
        let mut f_g = s_g.clone();
        remove_matches(&mut i_g, |r| r.is_float());
        remove_matches(&mut f_g, |r| r.is_usual());
        // 分别着色
        let (i_colors, i_spills) =
            ilp_alloc(&i_g, &free_i_colors.iter().cloned().collect(), costs)?;
        let (f_colors, f_spills) =
            ilp_alloc(&f_g, &free_f_colors.iter().cloned().collect(), costs)?;

        colors.extend(i_colors);
        colors.extend(f_colors);
        spills.extend(i_spills);
        spills.extend(f_spills);
    }

    while let Some(r) = later_to_color.pop_back() {
        let interred_colors: FxHashSet<Reg> = physical_inters(graph, Some(&colors), &r);
        if let Some(color) = select_free_color(free_i_colors, free_f_colors, &r, &interred_colors) {
            colors.insert(r, color);
        } else {
            spills.insert(r);
        }
    }

    Ok((colors, spills))
}

#[cfg(feature = "ilp_alloc")]
pub fn ilp_alloc(
    g: &FxHashMap<Reg, FxHashSet<Reg>>,
    free_colors: &FxHashSet<Reg>,
    costs: &FxHashMap<Reg, usize>,
) -> Result<(FxHashMap<Reg, Reg>, FxHashSet<Reg>)> {
    use z3::{
        ast::{Ast, Int},
        Config, Context, Optimize,
    };

    #[inline]
    fn var_color_name(node: Reg, color: Reg) -> String {
        format!("{}_{}", node.gen_asm(), color.gen_asm())
    }
    #[inline]
    fn var_spill_name(node: Reg) -> String {
        format!("{}_spill", node.gen_asm())
    }
    #[inline]
    fn var_color(ctx: &z3::Context, node: Reg, color: Reg) -> z3::ast::Int {
        z3::ast::Int::new_const(ctx, var_color_name(node, color))
    }
    #[inline]
    fn var_spill(ctx: &z3::Context, node: Reg) -> z3::ast::Int {
        z3::ast::Int::new_const(ctx, var_spill_name(node))
    }
    #[inline]
    fn vars_sum<'a>(ctx: &'a z3::Context, vars: Vec<z3::ast::Int<'a>>) -> z3::ast::Int<'a> {
        vars.into_iter()
            .fold(z3::ast::Int::from_i64(ctx, 0), |acc, x| {
                z3::ast::Int::add(ctx, &[&acc, &x])
            })
    }

    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let opt = Optimize::new(&ctx);

    // 给每个node 对应变量设置0-1约束
    for n in g.keys() {
        for color in free_colors {
            opt.assert(&var_color(&ctx, *n, *color).ge(&Int::from_i64(&ctx, 0)));
            opt.assert(&var_color(&ctx, *n, *color).le(&Int::from_i64(&ctx, 1)));
        }
        opt.assert(&var_spill(&ctx, *n).ge(&Int::from_i64(&ctx, 0)));
        opt.assert(&var_spill(&ctx, *n).le(&Int::from_i64(&ctx, 1)));
    }

    // 建立着色约束,每个寄存器要么着色,要么溢出, 同时相邻的寄存器不能够着相同的颜色
    for (node_id, neighbors) in g.iter() {
        // 对于同一个node, 只能在colors和spill中选择一个
        let mut vars: Vec<Int> = free_colors
            .iter()
            .map(|color| var_color(&ctx, *node_id, *color))
            .collect();
        vars.push(var_spill(&ctx, *node_id));
        let sum = vars_sum(&ctx, vars);
        opt.assert(&sum._eq(&Int::from_i64(&ctx, 1)));

        // 对于相邻的node, 如果颜色相同,则冲突,则意味着他们不能够同时选择一种颜色,
        for color in free_colors {
            let vars: Vec<Int<'_>> = neighbors
                .iter()
                .map(|neighbor| var_color(&ctx, *neighbor, *color))
                .collect();
            let sum = vars
                .into_iter()
                .fold(var_color(&ctx, *node_id, *color), |acc, x| {
                    Int::add(&ctx, &[&acc, &x])
                });
            opt.assert(&sum.le(&Int::from_i64(&ctx, 1)));
            opt.assert(&sum.ge(&Int::from_i64(&ctx, 0)));
        }
    }

    // 建立 目标代价函数
    let var_cost = vars_sum(
        &ctx,
        g.keys()
            .map(|node_id| {
                let cost = costs.get(node_id).unwrap_or(&1);
                Int::mul(
                    &ctx,
                    &[
                        &var_spill(&ctx, *node_id),
                        &Int::from_i64(&ctx, *cost as i64),
                    ],
                )
            })
            .collect(),
    );
    opt.minimize(&var_cost);

    let mut colors: FxHashMap<Reg, Reg> = FxHashMap::default();
    let mut spills: FxHashSet<Reg> = FxHashSet::default();
    let stat = opt.check(&[]);

    if stat == z3::SatResult::Sat {
        let model = opt.get_model().unwrap();
        // 查看所有计算出的值
        // for n in g.keys() {
        //     for color in free_colors {
        //         let var = var_color(&ctx, *n, *color);
        //         let value = model.eval(&var, true).unwrap().as_i64().unwrap();
        //         println!("{} = {}", var_color_name(*n, *color), value);
        //     }
        //     let var = var_spill(&ctx, *n);
        //     let value = model.eval(&var, true).unwrap().as_i64().unwrap();
        //     println!("{} = {}", var_spill_name(*n), value);
        // }

        for (reg, _) in g.iter() {
            for color in free_colors {
                let var = var_color(&ctx, *reg, *color);
                if model.eval(&var, true).unwrap().as_i64().unwrap() == 1 {
                    colors.insert(*reg, *color);
                    break;
                }
            }
            if !colors.contains_key(reg) {
                spills.insert(*reg);
            }
        }
    } else {
        bail!("No solution found");
    }
    Ok((colors, spills))
}
