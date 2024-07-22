use std::usize;

use crate::utils::mem::{ObjPool, ObjPtr};

use self::domin::make_domin;

use super::*;

type LoopPtr = ObjPtr<LoopTree>;

#[allow(dead_code)]
pub struct LoopTree {
    pub head: BBPtr,
    pub blocks: HashSet<BBPtr>,
    pub parent_loop: Option<LoopPtr>,
    pub sub_loops: Vec<LoopPtr>,
}

#[allow(dead_code)]
impl LoopTree {
    pub fn is_in_cur_loop(&self, bb: &BBPtr) -> bool {
        self.blocks.contains(bb)
    }

    pub fn is_in_loop(&self, bb: &BBPtr) -> bool {
        self.blocks.contains(bb) || self.sub_loops.iter().any(|lo| lo.is_in_loop(bb))
    }
}

#[allow(dead_code)]
pub struct LoopForest {
    pool: ObjPool<LoopTree>,
    // 只包含最外层循环，内部循环通过LoopPtr的sub_loops域访问
    pub forest: Vec<LoopPtr>,
}

#[allow(dead_code)]
impl LoopForest {
    // tarjan算法变体
    pub fn make_forest(func: FunPtr) -> Option<LoopForest> {
        let domin = make_domin(func);
        let mut stack;
        if let Some(x) = func.entry {
            stack = vec![x];
        } else {
            return None;
        }

        let mut loop_tree_meta = Vec::new();
        let mut id_map = HashMap::new();

        while let Some(bb) = stack.pop() {
            if let std::collections::hash_map::Entry::Vacant(e) = id_map.entry(bb) {
                // 第一次遍历该结点

                // 初始化
                e.insert(HashSet::from([loop_tree_meta.len()]));
                loop_tree_meta.push((bb, HashSet::from([bb]), None::<BBPtr>));
                stack.push(bb);

                // dfs
                if let Some(next) = bb.get_succ_bb().first() {
                    if !id_map.contains_key(next) {
                        stack.push(*next);
                    }
                }
            } else if bb
                .get_succ_bb()
                .iter()
                .all(|next| id_map.contains_key(next))
            {
                // bb的所有分支均已访问

                // 合并下游bb序号集中小于当前bb的序号
                let cur_id = *id_map.get(&bb).unwrap().iter().next().unwrap();
                let cur_map: HashSet<usize> = bb
                    .get_succ_bb()
                    .iter()
                    .map(|x| id_map.get(x).unwrap().clone())
                    .reduce(|acc, e| &acc | &e)
                    .unwrap_or(HashSet::new())
                    .into_iter()
                    .filter(|x| *x <= cur_id && domin.is_dominate(loop_tree_meta[*x].0, bb))
                    .collect();

                // 获取当前序号集中最大两个数（如有）
                let mut max_two = [-1, -1];
                cur_map.iter().for_each(|&x| {
                    let x = x as i32;
                    if x > max_two[0] {
                        max_two[1] = max_two[0];
                        max_two[0] = x;
                    } else if x > max_two[1] {
                        max_two[1] = x;
                    }
                });
                id_map.insert(bb, cur_map).unwrap();
                id_map.get_mut(&bb).unwrap().insert(cur_id);

                // 最大的数为当前循环的head bb的id
                // 次之为父循环的head bb的id
                // 依次类推
                match max_two {
                    [-1, -1] => {}
                    [x, -1] => {
                        loop_tree_meta[x as usize].1.insert(bb);
                    }
                    [x, y] => {
                        loop_tree_meta[x as usize].1.insert(bb);
                        if x as usize == cur_id {
                            loop_tree_meta[x as usize].2 = Some(loop_tree_meta[y as usize].0);
                        }
                    }
                }
            } else {
                // 当前bb有双分支，且第二个分支还未访问
                stack.push(bb);
                stack.push(bb.get_succ_bb()[1]);
            }
        }

        let mut forest = LoopForest {
            pool: ObjPool::new(),
            forest: Vec::new(),
        };
        let mut forest_map = HashMap::new();

        for (head, blocks, parent_loop) in loop_tree_meta.into_iter() {
            let loop_ptr = forest.pool.alloc(LoopTree {
                head,
                blocks,
                parent_loop: parent_loop.and_then(|x| forest_map.get(&x).cloned()),
                sub_loops: Vec::new(),
            });
            if let Some(par) = loop_ptr.parent_loop {
                forest_map
                    .get_mut(&par.head)
                    .iter_mut()
                    .for_each(|x| x.sub_loops.push(loop_ptr));
            } else {
                forest.forest.push(loop_ptr);
            };
            forest_map.insert(head, loop_ptr);
        }

        forest.forest.retain(|x| {
            x.blocks.len() > 1
                || !x.sub_loops.is_empty()
                || x.head.get_succ_bb().iter().any(|succ| *succ == x.head)
        });

        Some(forest)
    }
}

#[cfg(test)]
mod test_loop {
    use crate::middle::ir::IRBuilder;
    use std::iter::zip;

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

    fn gen_forest(bb_vec: Vec<[i32; 2]>) -> (IRBuilder, Option<LoopForest>, Vec<BBPtr>) {
        let mut pool = IRBuilder::new();
        let (func, bb_vec) = gen_graph(&mut pool, bb_vec);
        let forest = LoopForest::make_forest(func);
        (pool, forest, bb_vec)
    }

    #[test]
    fn one_loop() {
        let (_pool, forest, bb_vec) = gen_forest(vec![[1, -1], [2, 3], [1, -1], [-1, -1]]);
        assert!(forest.is_some());

        let forest = forest.unwrap();
        assert_eq!(forest.forest.len(), 1);

        let lo = forest.forest.first().unwrap();
        assert!(!lo.is_in_loop(&bb_vec[0]));
        assert!(lo.is_in_loop(&bb_vec[1]));
        assert!(lo.is_in_loop(&bb_vec[2]));
        assert!(!lo.is_in_loop(&bb_vec[3]));
    }

    #[test]
    fn two_loop() {
        let (_pool, forest, bb_vec) = gen_forest(vec![[1, -1], [0, 2], [3, -1], [2, 4], [-1, -1]]);
        assert!(forest.is_some());

        let forest = forest.unwrap();
        assert_eq!(forest.forest.len(), 2);

        let first = forest.forest[0];
        assert!(first.is_in_loop(&bb_vec[0]));
        assert!(first.is_in_loop(&bb_vec[1]));
        assert!(!first.is_in_loop(&bb_vec[2]));
        assert!(!first.is_in_loop(&bb_vec[4]));

        let second = forest.forest[1];
        assert!(second.is_in_loop(&bb_vec[2]));
        assert!(second.is_in_loop(&bb_vec[3]));
        assert!(!second.is_in_loop(&bb_vec[4]));
    }

    #[test]
    fn conponent_loop() {
        let (_pool, forest, bb_vec) = gen_forest(vec![
            [1, -1],
            [0, 2],
            [0, 3],
            [4, -1],
            [5, -1],
            [4, 6],
            [4, -1],
        ]);

        assert!(forest.is_some());
        let forest = forest.unwrap();
        assert_eq!(forest.forest.len(), 2);

        let first = forest.forest[0];
        assert_eq!(first.blocks.len(), 3);
        assert!(first.is_in_loop(&bb_vec[0]));
        assert!(first.is_in_loop(&bb_vec[1]));
        assert!(first.is_in_loop(&bb_vec[2]));

        let second = forest.forest[1];
        assert_eq!(second.blocks.len(), 3);
        assert!(second.is_in_loop(&bb_vec[4]));
        assert!(second.is_in_loop(&bb_vec[5]));
        assert!(second.is_in_loop(&bb_vec[6]));
    }

    #[test]
    fn branch_loop() {
        let (_pool, forest, bb_vec) = gen_forest(vec![[1, 2], [3, -1], [3, -1], [0, -1]]);

        assert!(forest.is_some());
        let forest = forest.unwrap();
        assert_eq!(forest.forest.len(), 1);

        let lo = forest.forest[0];
        assert_eq!(lo.blocks.len(), 4);
        assert_eq!(lo.sub_loops.len(), 0);
        assert!(lo.parent_loop.is_none());
        assert!(lo.is_in_cur_loop(&bb_vec[0]));
        assert!(lo.is_in_cur_loop(&bb_vec[1]));
        assert!(lo.is_in_cur_loop(&bb_vec[2]));
        assert!(lo.is_in_cur_loop(&bb_vec[3]));
    }

    #[test]
    fn nested_loop() {
        let (_pool, forest, bb_vec) = gen_forest(vec![[1, -1], [2, -1], [0, 1]]);
        assert!(forest.is_some());
        let forest = forest.unwrap();
        assert_eq!(forest.forest.len(), 1);
        let lo = forest.forest[0];
        assert_eq!(lo.blocks.len(), 1);
        assert_eq!(lo.head, bb_vec[0]);
        assert!(lo.is_in_cur_loop(&bb_vec[0]));

        assert_eq!(lo.sub_loops.len(), 1);
        let sub = lo.sub_loops[0];
        assert_eq!(sub.blocks.len(), 2);
        assert_eq!(sub.head, bb_vec[1]);
        assert!(sub.is_in_cur_loop(&bb_vec[1]));
        assert!(sub.is_in_cur_loop(&bb_vec[2]));
    }

    #[test]
    fn nested_branch_loop() {
        let (_pool, forest, bb_vec) =
            gen_forest(vec![[1, 5], [2, 3], [4, -1], [4, -1], [1, 0], [-1, -1]]);
        assert!(forest.is_some());
        let forest = forest.unwrap();
        assert_eq!(forest.forest.len(), 1);
        let lo = forest.forest[0];
        assert_eq!(lo.head, bb_vec[0]);
        assert!(lo.is_in_cur_loop(&bb_vec[0]));
        assert!(lo.is_in_loop(&bb_vec[1]));
        assert!(lo.is_in_loop(&bb_vec[2]));
        assert!(lo.is_in_loop(&bb_vec[3]));
        assert!(lo.is_in_loop(&bb_vec[4]));
        assert_eq!(lo.blocks.len(), 1);
        assert_eq!(lo.sub_loops.len(), 1);

        let sub = lo.sub_loops[0];
        assert_eq!(sub.blocks.len(), 4);
        assert!(sub.is_in_loop(&bb_vec[1]));
        assert!(sub.is_in_loop(&bb_vec[2]));
        assert!(sub.is_in_loop(&bb_vec[3]));
        assert!(sub.is_in_loop(&bb_vec[4]));
    }
}
