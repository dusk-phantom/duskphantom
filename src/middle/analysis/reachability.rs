use std::collections::HashSet;

use crate::middle::ir::{BBPtr, FunPtr};

pub struct Reachability {
    reachable: HashSet<BBPtr>,
}

impl Reachability {
    pub fn new(func: FunPtr) -> Self {
        let mut reachable = HashSet::new();
        for bb in func.dfs_iter() {
            reachable.insert(bb);
        }
        Self { reachable }
    }
}
