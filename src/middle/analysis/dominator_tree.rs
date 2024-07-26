use std::collections::{HashMap, HashSet};

use crate::middle::ir::{BBPtr, FunPtr};

pub struct DominatorTree {
    func: FunPtr,
    dom_map: HashMap<BBPtr, HashSet<BBPtr>>,
    idom_map: Option<HashMap<BBPtr, BBPtr>>,
    df_map: Option<HashMap<BBPtr, HashSet<BBPtr>>>,
}

#[allow(unused)]
impl DominatorTree {
    pub fn new(fun: FunPtr) -> Self {
        DominatorTree {
            func: fun,
            dom_map: HashMap::new(),
            idom_map: None,
            df_map: None,
        }
    }

    pub fn is_dominate(&mut self, dominator: BBPtr, dominatee: BBPtr) -> bool {
        self.get_dominator(dominatee).contains(&dominator)
    }

    pub fn get_dominator(&mut self, dominatee: BBPtr) -> HashSet<BBPtr> {
        match self.dom_map.get(&dominatee) {
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

    pub fn get_dominatee(&mut self, dominator: BBPtr) -> Vec<BBPtr> {
        self.get_idom_map()
            .iter()
            .filter_map(
                |(bb, idom)| {
                    if *idom == dominator {
                        Some(*bb)
                    } else {
                        None
                    }
                },
            )
            .collect()
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
                let entry = self.func.entry.unwrap();
                self.idom_map = Some(get_idom_map(entry));
                self.idom_map.as_ref().unwrap()
            }
        }
    }

    fn get_df_map(&mut self) -> &HashMap<BBPtr, HashSet<BBPtr>> {
        match self.df_map {
            Some(ref df) => df,
            None => {
                let func = self.func;
                let idoms = self.get_idom_map();
                self.df_map = Some(get_df_map(func, idoms));
                self.df_map.as_ref().unwrap()
            }
        }
    }
}

/// Get dominance frontiers of each basic block in the function
#[allow(unused)]
fn get_df_map(fun: FunPtr, idoms: &HashMap<BBPtr, BBPtr>) -> HashMap<BBPtr, HashSet<BBPtr>> {
    let mut df = HashMap::new();
    for bb in fun.dfs_iter() {
        for pred in bb.get_pred_bb() {
            let mut runner = *pred;

            // Hop up from each predecessor until runner is a dominator of bb
            // For non-entry block, the first hit must be it's immediate dominator
            // For entry block, the first hit must be itself, as doms(entry) = { entry }
            while runner != idoms.get(&bb).copied().unwrap_or(bb) {
                df.entry(runner).or_insert(HashSet::new()).insert(bb);

                // Only update runner if it's not dead block
                if let Some(new_runner) = idoms.get(&runner) {
                    runner = *new_runner;
                } else {
                    break;
                }
            }
        }
    }
    df
}

/// Get immediate dominators of each basic block in the function
#[allow(unused)]
fn get_idom_map(entry: BBPtr) -> HashMap<BBPtr, BBPtr> {
    let mut idoms = HashMap::new();

    /// Calculate postorder with dfs
    fn dfs_postorder(
        current_bb: BBPtr,
        visited: &mut HashSet<BBPtr>,
        postorder_map: &mut HashMap<BBPtr, i32>,
        postorder_array: &mut Vec<BBPtr>,
    ) {
        if visited.contains(&current_bb) {
            return;
        }
        visited.insert(current_bb);
        for succ in current_bb.get_succ_bb() {
            dfs_postorder(*succ, visited, postorder_map, postorder_array);
        }
        postorder_map.insert(current_bb, postorder_map.len() as i32);
        postorder_array.push(current_bb);
    }
    let mut postorder_map = HashMap::new();
    let mut postorder_array = Vec::new();
    dfs_postorder(
        entry,
        &mut HashSet::new(),
        &mut postorder_map,
        &mut postorder_array,
    );

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

    // Calculate idom with reverse postorder
    for current_bb in postorder_array.iter().rev() {
        if *current_bb == entry {
            continue;
        }
        let mut new_idom = None;
        for pred in current_bb.get_pred_bb() {
            if idoms.contains_key(pred) {
                if let Some(idom) = new_idom {
                    new_idom = Some(intersect(*pred, idom, &postorder_map, &idoms));
                } else {
                    new_idom = Some(*pred);
                }
            }
        }
        idoms.insert(*current_bb, new_idom.unwrap_or(entry));
    }

    // Return idoms
    idoms
}

#[cfg(test)]
pub mod tests_dominator_tree {
    use super::*;
    use crate::middle::ir::IRBuilder;
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
                pool.new_function("no_name".to_string(), crate::middle::ir::ValueType::Void);
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
