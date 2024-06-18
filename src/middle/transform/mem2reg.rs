use core::panic;
use std::collections::{HashMap, HashSet};

use crate::middle::ir::{instruction::InstType, BBPtr, InstPtr, Operand};

type PhiArg = (BBPtr, Operand);
type PhiArgs = Vec<PhiArg>;

/// The mem2reg pass
#[allow(unused)]
pub fn mem2reg(entry: BBPtr) {
    let phi_positions: HashMap<InstPtr, HashSet<BBPtr>> = get_variable_to_phi_insertion(entry);
    let mut phi_for_block: HashMap<BBPtr, HashSet<InstPtr>> = HashMap::new();

    // populate `phi_for_block` from `phi_positions`
    for (variable, positions) in phi_positions.iter() {
        for position in positions.iter() {
            phi_for_block
                .entry(*position)
                .or_default()
                .insert(*variable);
        }
    }

    // for each "phi" insert position, decide the value for each argument
    fn decide_variable_value(
        variable: InstPtr,
        current_variable_value: &mut [HashMap<InstPtr, PhiArg>],
    ) -> (BBPtr, Operand) {
        for frame in current_variable_value.iter().rev() {
            if let Some(value) = frame.get(&variable) {
                return value.clone();
            }
        }
        panic!("variable value not found");
    }
    fn decide_values_start_from(
        entry: BBPtr,
        visited: &mut HashSet<BBPtr>,
        current_variable_value: &mut [HashMap<InstPtr, PhiArg>],
        block_to_phi_insertion: &mut HashMap<BBPtr, HashMap<InstPtr, PhiArgs>>,
    ) {
        // decide value for each "phi" instruction to add
        for (variable, phi_args) in block_to_phi_insertion
            .get_mut(&entry)
            .unwrap_or(&mut HashMap::new())
            .iter_mut()
        {
            let value = decide_variable_value(*variable, current_variable_value);
            phi_args.push(value);
            current_variable_value
                .last_mut()
                .unwrap()
                .insert(*variable, (entry, Operand::Constant(0.into())));
        }
    }
}

/// Get all "phi" to insert for each basic block
/// Returns: basic_block -> (alloca -> [(from_block, operand)])
#[allow(unused)]
fn get_block_to_phi_insertion(
    entry: BBPtr,
) -> HashMap<BBPtr, HashMap<InstPtr, Vec<(BBPtr, Operand)>>> {
    let phi_insert_positions = get_variable_to_phi_insertion(entry);
    let mut block_to_phi_insertion: HashMap<BBPtr, HashMap<InstPtr, Vec<(BBPtr, Operand)>>> =
        HashMap::new();
    for (variable, positions) in phi_insert_positions.iter() {
        for position in positions.iter() {
            block_to_phi_insertion
                .entry(*position)
                .or_default()
                .insert(*variable, vec![]);
        }
    }
    block_to_phi_insertion
}

/// Get places to insert "phi" instructions for each "alloca" instruction
#[allow(unused)]
fn get_variable_to_phi_insertion(entry: BBPtr) -> HashMap<InstPtr, HashSet<BBPtr>> {
    let mut phi_positions = HashMap::new();
    let df = get_dominance_frontiers(entry);

    // for each "store", make insert position at the beginning of the dominance frontier
    fn dfs_insertion(
        current_bb: BBPtr,
        visited: &mut HashSet<BBPtr>,
        df: &HashMap<BBPtr, HashSet<BBPtr>>,
        phi_positions: &mut HashMap<InstPtr, HashSet<BBPtr>>,
    ) {
        if visited.contains(&current_bb) {
            return;
        }
        visited.insert(current_bb);
        for inst in current_bb.iter() {
            if inst.get_type() == InstType::Store {
                let store = inst;
                let store_operands = store.get_operand();
                let store_ptr = &store_operands[1];

                // only insert "phi" when store destination is a constant pointer
                if let Operand::Instruction(inst) = store_ptr {
                    if inst.get_type() == InstType::Alloca {
                        for df_bb in df.get(&current_bb).unwrap_or(&HashSet::new()).iter() {
                            phi_positions.entry(*inst).or_default().insert(*df_bb);
                        }
                    }
                }
            }
        }
        for succ in current_bb.get_succ_bb() {
            dfs_insertion(*succ, visited, df, phi_positions);
        }
    }
    dfs_insertion(entry, &mut HashSet::new(), &df, &mut phi_positions);

    // return result
    phi_positions
}

/// Get dominance frontiers of each basic block in the function
#[allow(unused)]
fn get_dominance_frontiers(entry: BBPtr) -> HashMap<BBPtr, HashSet<BBPtr>> {
    let idoms = get_immediate_dominators(entry);
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
fn get_immediate_dominators(entry: BBPtr) -> HashMap<BBPtr, BBPtr> {
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
    dfs_postorder(
        entry,
        &mut HashSet::new(),
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
        let idoms = get_immediate_dominators(entry);

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
        let df = get_dominance_frontiers(entry);

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
        let phi_positions = get_variable_to_phi_insertion(entry);

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
        let phi_positions = get_variable_to_phi_insertion(entry);

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
