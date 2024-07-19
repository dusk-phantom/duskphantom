use std::usize;

use crate::utils::mem::{ObjPool, ObjPtr};

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
    pub fn new() -> Self {
        LoopForest {
            pool: ObjPool::new(),
            forest: Vec::new(),
        }
    }

    pub fn make_forest(&mut self, func: FunPtr) -> Option<LoopForest> {
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
                    stack.push(*next);
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
                    .map(|x| id_map.get(x).map_or(HashSet::new(), |x| x.clone()))
                    .reduce(|acc, e| &acc | &e)
                    .unwrap_or(HashSet::new())
                    .into_iter()
                    .filter(|x| *x <= cur_id)
                    .collect();

                // 获取当前序号集中最大两个数（如有）
                let mut max_two = [-1, -1];
                cur_map.iter().for_each(|&x| {
                    let x = x as i32;
                    if x > max_two[1] {
                        if x > max_two[0] {
                            max_two[1] = max_two[0];
                            max_two[0] = x;
                        } else {
                            max_two[1] = x;
                        }
                    }
                });

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
                        loop_tree_meta[x as usize].2 = Some(loop_tree_meta[y as usize].0);
                    }
                }
            } else {
                // 当前bb有双分支，且第二个分支还未访问
                stack.push(bb);
                stack.push(bb.get_succ_bb()[1]);
            }
        }

        let mut forest = LoopForest::new();
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
            }
            forest_map.insert(head, loop_ptr);
        }

        Some(forest)
    }
}
