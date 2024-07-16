use crate::backend::{checker, BBIter};

use super::*;
pub struct Domin {
    checker: HashMap<BBPtr, HashSet<BBPtr>>,
}

impl Domin {
    pub fn is_dominate(&self, dominator: BBPtr, dominatee: BBPtr) -> bool {
        if let Some(dom) = self.checker.get(&dominatee) {
            dom.contains(&dominator)
        } else {
            false
        }
    }

    pub fn get_dominator(&self, dominatee: BBPtr) -> Vec<BBPtr> {
        self.checker
            .get(&dominatee)
            .map_or(vec![], |x| x.clone().into())
    }

    pub fn get_dominatee(&self, dominator: BBPtr) -> Vec<BBPtr> {
        self.checker
            .iter()
            .filter_map(|(bb, hm)| {
                if hm.contains(&dominator) {
                    Some(bb)
                } else {
                    None
                }
            })
            .collect()
    }
}

pub fn make_domin(func: FunPtr) -> Domin {
    let mut checker = HashMap::new();
    func.bfs_iter().for_each(|bb| {
        checker.insert(
            bb,
            bb.get_pred_bb()
                .iter()
                .filter_map(|up_bb| checker.get(up_bb))
                .reduce(|acc, e| acc.intersection(e))
                .map_or(HashSet::from([bb]), |acc| {
                    acc.intersection(&HashSet::from([bb]))
                }),
        );
    });
}
