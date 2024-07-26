use std::collections::{HashMap, HashSet};

use crate::middle::ir::BBPtr;

pub struct DominatorTree {
    entry: BBPtr,
    dom_map: HashMap<BBPtr, HashSet<BBPtr>>,
    idom_map: Option<HashMap<BBPtr, BBPtr>>,
    df_map: Option<HashMap<BBPtr, HashSet<BBPtr>>>,
}

#[allow(unused)]
impl DominatorTree {
    pub fn new(entry: BBPtr) -> Self {
        DominatorTree {
            entry,
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
                self.idom_map = Some(get_idom_map(self.entry));
                self.idom_map.as_ref().unwrap()
            }
        }
    }

    fn get_df_map(&mut self) -> &HashMap<BBPtr, HashSet<BBPtr>> {
        match self.df_map {
            Some(ref df) => df,
            None => {
                let entry = self.entry;
                let idoms = self.get_idom_map();
                self.df_map = Some(get_df_map(entry, idoms));
                self.df_map.as_ref().unwrap()
            }
        }
    }
}

/// Get dominance frontiers of each basic block in the function
#[allow(unused)]
fn get_df_map(entry: BBPtr, idoms: &HashMap<BBPtr, BBPtr>) -> HashMap<BBPtr, HashSet<BBPtr>> {
    let mut df = HashMap::new();

    /// Calculate dominance frontiers
    for (bb, idom) in idoms.iter() {
        if bb == idom {
            continue;
        }
        for pred in bb.get_pred_bb() {
            let mut runner = *pred;
            while runner != idoms[bb] {
                df.entry(runner).or_insert(HashSet::new()).insert(*bb);

                // Only update runner if it's not dead block
                if let Some(new_runner) = idoms.get(&runner) {
                    runner = *new_runner;
                } else {
                    break;
                }
            }
        }
    }

    // Return dominance frontiers
    df
}

/// Get immediate dominators of each basic block in the function
#[allow(unused)]
fn get_idom_map(entry: BBPtr) -> HashMap<BBPtr, BBPtr> {
    let mut idoms = HashMap::new();
    idoms.insert(entry, entry);

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
        idoms.insert(*current_bb, new_idom.unwrap());
    }

    // Return idoms
    idoms
}

#[cfg(test)]
pub mod tests_dominator_tree {
    use super::*;
    use crate::middle::Program;

    #[test]
    fn test_get_idom_map() {
        let mut program = Program::new();

        // Construct a nested if-else graph
        let mut entry = program.mem_pool.new_basicblock("entry".to_string());
        let mut then = program.mem_pool.new_basicblock("then".to_string());
        let mut then_then = program.mem_pool.new_basicblock("then_then".to_string());
        let mut then_alt = program.mem_pool.new_basicblock("then_alt".to_string());
        let mut alt = program.mem_pool.new_basicblock("alt".to_string());
        let end = program.mem_pool.new_basicblock("end".to_string());
        entry.set_true_bb(then);
        entry.set_false_bb(alt);
        then.set_true_bb(then_then);
        then.set_false_bb(then_alt);
        then_then.set_true_bb(end);
        then_alt.set_true_bb(end);
        alt.set_true_bb(end);

        // Calculate idoms
        let idoms = get_idom_map(entry);

        // Check if idoms are correct
        assert_eq!(idoms[&entry].name, entry.name);
        assert_eq!(idoms[&then].name, entry.name);
        assert_eq!(idoms[&then_then].name, then.name);
        assert_eq!(idoms[&then_alt].name, then.name);
        assert_eq!(idoms[&alt].name, entry.name);
        assert_eq!(idoms[&end].name, entry.name);
    }

    #[test]
    fn test_get_df_map() {
        let mut program = Program::new();

        // Construct a nested if-else graph
        let mut entry = program.mem_pool.new_basicblock("entry".to_string());
        let mut then = program.mem_pool.new_basicblock("then".to_string());
        let mut then_then = program.mem_pool.new_basicblock("then_then".to_string());
        let mut then_alt = program.mem_pool.new_basicblock("then_alt".to_string());
        let mut alt = program.mem_pool.new_basicblock("alt".to_string());
        let end = program.mem_pool.new_basicblock("end".to_string());
        entry.set_true_bb(then);
        entry.set_false_bb(alt);
        then.set_true_bb(then_then);
        then.set_false_bb(then_alt);
        then_then.set_true_bb(end);
        then_alt.set_true_bb(end);
        alt.set_true_bb(end);

        // Calculate df
        let idoms = get_idom_map(entry);
        let df = get_df_map(entry, &idoms);

        // Check if df lengths are correct
        assert_eq!(df.len(), 4);
        assert_eq!(df[&then].len(), 1);
        assert_eq!(df[&then_then].len(), 1);
        assert_eq!(df[&then_alt].len(), 1);
        assert_eq!(df[&alt].len(), 1);

        // Check if df contents are correct
        assert!(df[&then].contains(&end));
        assert!(df[&then_then].contains(&end));
        assert!(df[&then_alt].contains(&end));
        assert!(df[&alt].contains(&end));
    }
}
