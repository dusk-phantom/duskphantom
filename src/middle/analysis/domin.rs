use super::*;

#[allow(dead_code)]
pub struct Domin {
    checker: HashMap<BBPtr, HashSet<BBPtr>>,
}

impl Domin {
    #[allow(dead_code)]
    pub fn is_dominate(&self, dominator: BBPtr, dominatee: BBPtr) -> bool {
        if let Some(dom) = self.checker.get(&dominatee) {
            dom.contains(&dominator)
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub fn get_dominator(&self, dominatee: BBPtr) -> Vec<BBPtr> {
        self.checker
            .get(&dominatee)
            .map_or(vec![], |x| x.iter().copied().collect())
    }

    #[allow(dead_code)]
    pub fn get_dominatee(&self, dominator: BBPtr) -> Vec<BBPtr> {
        self.checker
            .iter()
            .filter_map(|(bb, hm)| {
                if hm.contains(&dominator) {
                    Some(*bb)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[allow(dead_code)]
pub fn make_domin(func: FunPtr) -> Domin {
    let mut checker = HashMap::new();
    func.bfs_iter().for_each(|bb| {
        checker.insert(
            bb,
            bb.get_pred_bb()
                .iter()
                .filter_map(|up_bb| checker.get(up_bb).cloned())
                .reduce(|acc, e| &acc & &e)
                .map_or(HashSet::from([bb]), |acc| &acc | &(HashSet::from([bb]))),
        );
    });
    Domin { checker }
}

#[cfg(test)]
pub mod tests_domin {

    use std::iter::zip;

    use crate::middle::ir::{FunPtr, IRBuilder};

    use super::*;
    fn gen_graph(pool: &mut IRBuilder, down_stream: Vec<[i32; 2]>) -> (FunPtr, Vec<BBPtr>) {
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
        let mut func = pool.new_function("no_name".to_string(), crate::middle::ir::ValueType::Void);
        func.entry = Some(bb_vec[0]);
        func.exit = Some(bb_vec[bb_vec.len() - 1]);
        (func, bb_vec)
    }

    fn check_domin(bb_vec: &[BBPtr], domin: &Domin, i: usize, j: usize) -> bool {
        domin.is_dominate(bb_vec[i], bb_vec[j])
    }

    #[test]
    fn basic_test() {
        let mut pool = IRBuilder::new();
        let (func, bb_vec) = gen_graph(&mut pool, vec![[1, -1], [-1, -1]]);
        let domin = make_domin(func);
        assert!(domin.is_dominate(bb_vec[0], bb_vec[1]));
        assert!(!domin.is_dominate(bb_vec[1], bb_vec[0]));
        assert!(domin.is_dominate(bb_vec[0], bb_vec[0]));
        assert!(domin.is_dominate(bb_vec[1], bb_vec[1]));
    }

    #[test]
    fn no_back_edge() {
        let mut pool = IRBuilder::new();
        let (func, bb_vec) = gen_graph(
            &mut pool,
            vec![[1, 2], [3, -1], [3, 5], [4, -1], [5, -1], [-1, -1]],
        );
        let domin = make_domin(func);
        let check_t = |i, j| assert!(check_domin(&bb_vec, &domin, i, j));
        let check_f = |i, j| assert!(!check_domin(&bb_vec, &domin, i, j));

        // check self
        assert!(bb_vec.iter().all(|bb| domin.is_dominate(*bb, *bb)));

        check_t(0, 1);
        check_t(0, 2);
        check_t(0, 3);
        check_t(0, 4);
        check_t(0, 5);

        check_f(1, 3);

        check_f(2, 3);
        check_t(2, 5);

        check_t(3, 4);
        check_f(3, 5);

        check_f(4, 5);
    }

    #[test]
    fn back_edge() {
        let mut pool = IRBuilder::new();
        let (func, bb_vec) = gen_graph(&mut pool, vec![[1, -1], [2, 3], [1, 3], [-1, -1]]);
        let domin = make_domin(func);

        let check_t = |i, j| assert!(check_domin(&bb_vec, &domin, i, j));
        let check_f = |i, j| assert!(!check_domin(&bb_vec, &domin, i, j));

        check_t(0, 1);
        check_t(0, 2);
        check_t(0, 3);

        check_t(1, 2);
        check_t(1, 3);

        check_f(2, 1);
        check_f(2, 3);
    }
}
