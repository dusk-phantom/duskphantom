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

use std::collections::{HashMap, HashSet};

use crate::ir::{BBPtr, FunPtr};

pub struct DominatorTree {
    fun: FunPtr,
    dominator_map: HashMap<BBPtr, HashSet<BBPtr>>,
    dominatee_map: Option<HashMap<BBPtr, HashSet<BBPtr>>>,
    idom_map: Option<HashMap<BBPtr, BBPtr>>,
    postorder_map: Option<HashMap<BBPtr, i32>>,
    df_map: Option<HashMap<BBPtr, HashSet<BBPtr>>>,
}

#[allow(unused)]
impl DominatorTree {
    pub fn new(fun: FunPtr) -> Self {
        DominatorTree {
            fun,
            dominator_map: HashMap::new(),
            dominatee_map: None,
            idom_map: None,
            postorder_map: None,
            df_map: None,
        }
    }

    pub fn is_dominate(&mut self, dominator: BBPtr, dominatee: BBPtr) -> bool {
        self.get_dominator(dominatee).contains(&dominator)
    }

    pub fn get_dominator(&mut self, dominatee: BBPtr) -> HashSet<BBPtr> {
        match self.dominator_map.get(&dominatee) {
            Some(dom) => dom.clone(),
            None => {
                // Traverse up dominator tree
                let mut dom = HashSet::new();
                let mut cursor = dominatee;
                loop {
                    dom.insert(cursor);
                    match self.get_idom(cursor) {
                        Some(idom) => cursor = idom,
                        None => break,
                    }
                }
                dom
            }
        }
    }

    pub fn get_lca(&mut self, a: BBPtr, b: BBPtr) -> BBPtr {
        self.get_idom_map();
        let idoms = self.idom_map.as_ref().unwrap();
        let depths = self.postorder_map.as_ref().unwrap();
        intersect(a, b, depths, idoms)
    }

    pub fn get_dominatee(&mut self, dominator: BBPtr) -> HashSet<BBPtr> {
        self.get_dominatee_map()
            .get(&dominator)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_idom(&mut self, dominatee: BBPtr) -> Option<BBPtr> {
        self.get_idom_map().get(&dominatee).copied()
    }

    pub fn get_df(&mut self, dominator: BBPtr) -> HashSet<BBPtr> {
        self.get_df_map()
            .get(&dominator)
            .cloned()
            .unwrap_or_default()
    }

    fn get_idom_map(&mut self) -> &HashMap<BBPtr, BBPtr> {
        match self.idom_map {
            Some(ref idoms) => idoms,
            None => {
                self.calculate_idom();
                self.idom_map.as_ref().unwrap()
            }
        }
    }

    fn get_dominatee_map(&mut self) -> &HashMap<BBPtr, HashSet<BBPtr>> {
        match self.dominatee_map {
            Some(ref doms) => doms,
            None => {
                self.calculate_idom();
                self.dominatee_map.as_ref().unwrap()
            }
        }
    }

    fn get_df_map(&mut self) -> &HashMap<BBPtr, HashSet<BBPtr>> {
        match self.df_map {
            Some(ref df) => df,
            None => {
                self.calculate_df();
                self.df_map.as_ref().unwrap()
            }
        }
    }

    fn calculate_idom(&mut self) {
        let entry = self.fun.entry.unwrap();
        let mut idom_map = HashMap::new();
        let mut dominatee_map = HashMap::new();
        let mut postorder_map = HashMap::new();
        self.fun.po_iter().enumerate().for_each(|(i, bb)| {
            postorder_map.insert(bb, i as i32);
        });

        // Calculate idom with reverse postorder
        for current_bb in self.fun.rpo_iter() {
            if current_bb == entry {
                continue;
            }
            let mut new_idom = None;
            for pred in current_bb.get_pred_bb() {
                if idom_map.contains_key(pred) {
                    // Set idom as intersection of predecessors
                    if let Some(idom) = new_idom {
                        new_idom = Some(intersect(*pred, idom, &postorder_map, &idom_map));
                    } else {
                        new_idom = Some(*pred);
                    }
                } else if *pred == entry {
                    // If one of predecessor is entry, intersection is entry
                    new_idom = Some(entry);
                }
            }
            let new_idom = new_idom.unwrap_or(entry);
            idom_map.insert(current_bb, new_idom);
            dominatee_map
                .entry(new_idom)
                .or_insert(HashSet::new())
                .insert(current_bb);
        }

        // Assign idom map and dominatee map to self
        self.idom_map = Some(idom_map);
        self.postorder_map = Some(postorder_map);
        self.dominatee_map = Some(dominatee_map);
    }

    fn calculate_df(&mut self) {
        let fun = self.fun;
        let idoms = self.get_idom_map();
        let mut df_map = HashMap::new();
        for bb in fun.dfs_iter() {
            for pred in bb.get_pred_bb() {
                let mut runner = *pred;

                // Hop up from each predecessor until runner is a dominator of bb
                // For non-entry block, the first hit must be it's immediate dominator
                // For entry block, the first hit must be itself, as doms(entry) = { entry }
                while runner != idoms.get(&bb).copied().unwrap_or(bb) {
                    df_map.entry(runner).or_insert(HashSet::new()).insert(bb);

                    // Only update runner if it's not dead block
                    if let Some(new_runner) = idoms.get(&runner) {
                        runner = *new_runner;
                    } else {
                        break;
                    }
                }
            }
        }

        // Assign df map to self
        self.df_map = Some(df_map);
    }
}

/// Function to get lowest common ancestor of two basic blocks in the dominator tree
fn intersect(
    mut n: BBPtr,
    mut m: BBPtr,
    postorder_map: &HashMap<BBPtr, i32>,
    idoms: &HashMap<BBPtr, BBPtr>,
) -> BBPtr {
    while n != m {
        while postorder_map[&n] < postorder_map[&m] {
            n = idoms[&n];
        }
        while postorder_map[&m] < postorder_map[&n] {
            m = idoms[&m];
        }
    }
    n
}

#[cfg(test)]
pub mod tests_dominator_tree {
    use super::*;
    use crate::ir::IRBuilder;
    use std::iter::zip;

    struct TestContext {
        bb_vec: Vec<BBPtr>,
        dom_tree: DominatorTree,
    }

    impl TestContext {
        fn new(pool: &mut IRBuilder, down_stream: Vec<[i32; 2]>) -> Self {
            let bb_vec: Vec<BBPtr> = (0..down_stream.len())
                .map(|_| pool.new_basicblock("no_name".to_string()))
                .collect();
            for (mut bb, down) in zip(bb_vec.clone(), down_stream) {
                match down {
                    [-1, -1] => {}
                    [t, -1] => bb.set_true_bb(bb_vec[t as usize]),
                    [t, f] => {
                        bb.set_true_bb(bb_vec[t as usize]);
                        bb.set_false_bb(bb_vec[f as usize]);
                    }
                }
            }
            let mut fun =
                pool.new_function("no_name".to_string(), crate::ir::ValueType::Void);
            fun.entry = Some(bb_vec.first().cloned().unwrap());
            fun.exit = Some(bb_vec.last().cloned().unwrap());
            let dom_tree = DominatorTree::new(fun);
            Self { bb_vec, dom_tree }
        }

        /// Check if dom(i) == j.
        fn check_dominator(&mut self, i: usize, j: Vec<usize>) {
            let dom = self.dom_tree.get_dominator(self.bb_vec[i]);
            let expected: HashSet<BBPtr> = j.into_iter().map(|index| self.bb_vec[index]).collect();
            assert_eq!(dom, expected);
        }

        /// Check if idom(i) == j.
        fn check_idom(&mut self, i: usize, j: Option<usize>) {
            let idom = self.dom_tree.get_idom(self.bb_vec[i]);
            let expected = j.map(|index| self.bb_vec[index]);
            assert_eq!(idom, expected);
        }

        /// Check if df(i) == j.
        fn check_df(&mut self, i: usize, j: Vec<usize>) {
            let df = self.dom_tree.get_df(self.bb_vec[i]);
            let expected: HashSet<BBPtr> = j.into_iter().map(|index| self.bb_vec[index]).collect();
            assert_eq!(df, expected);
        }
    }

    #[test]
    fn basic_test() {
        // 0 ──► 1
        let mut pool = IRBuilder::new();
        let mut ctx = TestContext::new(&mut pool, vec![[1, -1], [-1, -1]]);
        ctx.check_dominator(0, vec![0]);
        ctx.check_dominator(1, vec![0, 1]);
        ctx.check_idom(0, None);
        ctx.check_idom(1, Some(0));
        ctx.check_df(0, vec![]);
        ctx.check_df(1, vec![]);
    }

    #[test]
    fn no_back_edge() {
        //    ┌─► 1 ──► 3 ──► 4
        //    │         ▲     │
        // 0 ─┤         │     │
        //    │         │     ▼
        //    └─► 2 ────┴───► 5
        let mut pool = IRBuilder::new();
        let mut ctx = TestContext::new(
            &mut pool,
            vec![[1, 2], [3, -1], [3, 5], [4, -1], [5, -1], [-1, -1]],
        );
        ctx.check_dominator(0, vec![0]);
        ctx.check_dominator(1, vec![0, 1]);
        ctx.check_dominator(2, vec![0, 2]);
        ctx.check_dominator(3, vec![0, 3]);
        ctx.check_dominator(4, vec![0, 3, 4]);
        ctx.check_dominator(5, vec![0, 5]);
        ctx.check_idom(0, None);
        ctx.check_idom(1, Some(0));
        ctx.check_idom(2, Some(0));
        ctx.check_idom(3, Some(0));
        ctx.check_idom(4, Some(3));
        ctx.check_idom(5, Some(0));
        ctx.check_df(0, vec![]);
        ctx.check_df(1, vec![3]);
        ctx.check_df(2, vec![3, 5]);
        ctx.check_df(3, vec![5]);
        ctx.check_df(4, vec![5]);
        ctx.check_df(5, vec![]);
    }

    #[test]
    fn back_edge() {
        //       ┌──────────┐
        //       │          │
        //       │  ┌─► 2 ──┤
        //       ▼  │       │
        // 0 ──► 1 ─┤       │
        //          │       │
        //          └─► 3 ◄─┘
        let mut pool = IRBuilder::new();
        let mut ctx = TestContext::new(&mut pool, vec![[1, -1], [2, 3], [1, 3], [-1, -1]]);
        ctx.check_dominator(0, vec![0]);
        ctx.check_dominator(1, vec![0, 1]);
        ctx.check_dominator(2, vec![0, 1, 2]);
        ctx.check_dominator(3, vec![0, 1, 3]);
        ctx.check_idom(0, None);
        ctx.check_idom(1, Some(0));
        ctx.check_idom(2, Some(1));
        ctx.check_idom(3, Some(1));
        ctx.check_df(0, vec![]);
        ctx.check_df(1, vec![1]);
        ctx.check_df(2, vec![1, 3]);
        ctx.check_df(3, vec![]);
    }

    #[test]
    fn branch_nested_loop() {
        //          ┌─► 2 ─┐
        //          │      │
        // 0 ──► 1 ─┤      ├─► 4 ─┐
        // ▲     ▲  │      │      │
        // │     │  └─► 3 ─┘      │
        // │     │                │
        // └─────┴────────────────┘
        let mut pool = IRBuilder::new();
        let mut ctx = TestContext::new(&mut pool, vec![[1, -1], [2, 3], [4, -1], [4, -1], [0, 1]]);
        ctx.check_dominator(0, vec![0]);
        ctx.check_dominator(1, vec![0, 1]);
        ctx.check_dominator(2, vec![0, 1, 2]);
        ctx.check_dominator(3, vec![0, 1, 3]);
        ctx.check_dominator(4, vec![0, 1, 4]);
        ctx.check_idom(0, None);
        ctx.check_idom(1, Some(0));
        ctx.check_idom(2, Some(1));
        ctx.check_idom(3, Some(1));
        ctx.check_idom(4, Some(1));
        ctx.check_df(0, vec![]);
        ctx.check_df(1, vec![0, 1]);
        ctx.check_df(2, vec![4]);
        ctx.check_df(3, vec![4]);
        ctx.check_df(4, vec![0, 1]);
    }
}
