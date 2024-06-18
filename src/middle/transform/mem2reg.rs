use std::collections::{HashMap, HashSet};

use crate::middle::ir::{instruction::InstType, BBPtr, InstPtr, Operand};

/// Get dominance frontiers of each basic block in the function
#[allow(unused)]
pub fn get_df(entry: BBPtr) -> HashMap<BBPtr, HashSet<BBPtr>> {
    let idoms = get_idoms(entry);
    let mut df = HashMap::new();

    // calculate dominance frontiers
    for (bb, idom) in idoms.iter() {
        if bb == idom {
            continue;
        }
        for pred in bb.get_pred_bb() {
            let mut runner = *pred;
            while runner != idoms[bb] {
                df.entry(runner).or_insert(HashSet::new()).insert(*bb);
                runner = idoms[&runner];
            }
        }
    }

    // return dominance frontiers
    df
}

/// Get immediate dominators of each basic block in the function
#[allow(unused)]
pub fn get_idoms(entry: BBPtr) -> HashMap<BBPtr, BBPtr> {
    let mut idoms = HashMap::new();
    idoms.insert(entry, entry);

    // calculate postorder with dfs
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
    let mut postorder_visited = HashSet::new();
    dfs_postorder(
        entry,
        &mut postorder_visited,
        &mut postorder_map,
        &mut postorder_array,
    );

    // function to get intersection of two nodes
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

    // calculate idom with reverse postorder
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

    // return idoms
    idoms
}

/// Get and remove all primitive "alloca" from function
#[allow(unused)]
pub fn consume_all_alloca(entry: BBPtr) -> Vec<InstPtr> {
    let mut alloca_list = Vec::new();

    // find and remove all "alloca" with dfs
    fn find_and_remove(
        current_bb: BBPtr,
        visited: &mut HashSet<BBPtr>,
        alloca_list: &mut Vec<InstPtr>,
    ) {
        if visited.contains(&current_bb) {
            return;
        }
        visited.insert(current_bb);
        let mut current_inst: Option<InstPtr> = if current_bb.is_empty() {
            None
        } else {
            Some(current_bb.get_first_inst())
        };
        while let Some(mut inst) = current_inst {
            if inst.get_type() == InstType::Alloca {
                alloca_list.push(inst);
                inst.remove_self();
            }
            current_inst = inst.get_next();
        }
        for succ in current_bb.get_succ_bb() {
            find_and_remove(*succ, visited, alloca_list);
        }
    }
    let mut visited = HashSet::new();
    find_and_remove(entry, &mut visited, &mut alloca_list);

    // return result
    alloca_list
}

/// Get places to insert "phi" instructions for each "alloca" instruction
#[allow(unused)]
pub fn phi_insert_positions(entry: BBPtr) -> HashMap<InstPtr, HashSet<BBPtr>> {
    let mut phi_positions = HashMap::new();
    let df = get_df(entry);

    // for each "store", make insert position at the beginning of the dominance frontier
    fn dfs_insert_positions(
        current_bb: BBPtr,
        visited: &mut HashSet<BBPtr>,
        df: &HashMap<BBPtr, HashSet<BBPtr>>,
        phi_positions: &mut HashMap<InstPtr, HashSet<BBPtr>>,
    ) {
        if visited.contains(&current_bb) {
            return;
        }
        visited.insert(current_bb);
        let mut current_inst: Option<InstPtr> = if current_bb.is_empty() {
            None
        } else {
            Some(current_bb.get_first_inst())
        };
        while let Some(mut inst) = current_inst {
            if inst.get_type() == InstType::Store {
                let store = inst;
                let store_operands = store.get_operand();
                let store_ptr = &store_operands[1];

                // only insert "phi" when store destination is a constant pointer
                if let Operand::Instruction(alloc) = store_ptr {
                    if alloc.get_type() == InstType::Alloca {
                        for df_bb in df.get(&current_bb).unwrap_or(&HashSet::new()).iter() {
                            phi_positions.entry(*alloc).or_default().insert(*df_bb);
                        }
                    }
                }
            }
            current_inst = inst.get_next();
        }
        for succ in current_bb.get_succ_bb() {
            dfs_insert_positions(*succ, visited, df, phi_positions);
        }
    }
    let mut visited = HashSet::new();
    dfs_insert_positions(entry, &mut visited, &df, &mut phi_positions);

    // return result
    phi_positions
}

#[cfg(test)]
pub mod tests_mem2reg {
    use super::*;
    use crate::middle::{ir::ValueType, Program};

    #[test]
    fn test_get_idoms() {
        let mut program = Program::new();

        // construct a nested if-else graph
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

        // calculate idoms
        let idoms = get_idoms(entry);

        // check if idoms are correct
        assert_eq!(idoms[&entry].name, entry.name);
        assert_eq!(idoms[&then].name, entry.name);
        assert_eq!(idoms[&then_then].name, then.name);
        assert_eq!(idoms[&then_alt].name, then.name);
        assert_eq!(idoms[&alt].name, entry.name);
        assert_eq!(idoms[&end].name, entry.name);
    }

    #[test]
    fn test_get_df() {
        let mut program = Program::new();

        // construct a nested if-else graph
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

        // calculate df
        let df = get_df(entry);

        // check if df lengths are correct
        assert_eq!(df.len(), 4);
        assert_eq!(df[&then].len(), 1);
        assert_eq!(df[&then_then].len(), 1);
        assert_eq!(df[&then_alt].len(), 1);
        assert_eq!(df[&alt].len(), 1);

        // check if df contents are correct
        assert!(df[&then].contains(&end));
        assert!(df[&then_then].contains(&end));
        assert!(df[&then_alt].contains(&end));
        assert!(df[&alt].contains(&end));
    }

    #[test]
    fn test_consume_all_alloca() {
        let mut program = Program::new();

        // construct a function with "alloca" instructions
        let mut entry = program.mem_pool.new_basicblock("entry".to_string());
        let alloca1 = program.mem_pool.get_alloca(ValueType::Int, 1);
        let alloca2 = program.mem_pool.get_alloca(ValueType::Int, 1);
        let alloca3 = program.mem_pool.get_alloca(ValueType::Int, 1);
        let store = program
            .mem_pool
            .get_store(Operand::Constant(1.into()), Operand::Instruction(alloca1));
        entry.push_back(alloca1);
        entry.push_back(alloca2);
        entry.push_back(alloca3);
        entry.push_back(store);

        // consume all "alloca" instructions
        let alloca_list = consume_all_alloca(entry);

        // check if all "alloca" instructions are consumed
        assert_eq!(alloca_list.len(), 3);
        assert_eq!(entry.get_first_inst().get_type(), InstType::Store);
    }

    #[test]
    fn test_phi_insert_positions_single() {
        let mut program = Program::new();

        // construct a function with "alloca" and "store" instructions
        let mut entry = program.mem_pool.new_basicblock("entry".to_string());
        let alloca1 = program.mem_pool.get_alloca(ValueType::Int, 1);
        let alloca2 = program.mem_pool.get_alloca(ValueType::Int, 1);
        let alloca3 = program.mem_pool.get_alloca(ValueType::Int, 1);
        let store1 = program
            .mem_pool
            .get_store(Operand::Constant(1.into()), Operand::Instruction(alloca1));
        let store2 = program
            .mem_pool
            .get_store(Operand::Constant(1.into()), Operand::Instruction(alloca2));
        let store3 = program
            .mem_pool
            .get_store(Operand::Constant(1.into()), Operand::Instruction(alloca3));
        entry.push_back(alloca1);
        entry.push_back(alloca2);
        entry.push_back(alloca3);
        entry.push_back(store1);
        entry.push_back(store2);
        entry.push_back(store3);

        // calculate df and phi insert positions
        let phi_positions = phi_insert_positions(entry);

        // check if phi insert positions are correct
        assert_eq!(phi_positions.len(), 0);
    }

    #[test]
    fn test_phi_insert_positions_nested() {
        let mut program = Program::new();

        // construct a nested if-else graph
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

        // construct "alloca" and "store" instructions
        let alloca1 = program.mem_pool.get_alloca(ValueType::Int, 1);
        let alloca2 = program.mem_pool.get_alloca(ValueType::Int, 1);
        let alloca3 = program.mem_pool.get_alloca(ValueType::Int, 1);
        let store1 = program
            .mem_pool
            .get_store(Operand::Constant(1.into()), Operand::Instruction(alloca1));
        let store2 = program
            .mem_pool
            .get_store(Operand::Constant(1.into()), Operand::Instruction(alloca2));
        let store3 = program
            .mem_pool
            .get_store(Operand::Constant(1.into()), Operand::Instruction(alloca3));
        then.push_back(alloca1);
        then.push_back(store1);
        then_alt.push_back(alloca2);
        then_alt.push_back(store2);
        alt.push_back(alloca3);
        alt.push_back(store3);

        // calculate phi insert positions
        let phi_positions = phi_insert_positions(entry);

        // check if phi insert positions are correct
        assert_eq!(phi_positions.len(), 3);
        assert_eq!(phi_positions[&alloca1].len(), 1);
        assert_eq!(phi_positions[&alloca2].len(), 1);
        assert_eq!(phi_positions[&alloca3].len(), 1);
        assert!(phi_positions[&alloca1].contains(&end));
        assert!(phi_positions[&alloca2].contains(&end));
        assert!(phi_positions[&alloca3].contains(&end));
    }
}
