use core::panic;
use std::collections::{HashMap, HashSet};

use crate::middle::{
    ir::{
        instruction::{downcast_mut, misc_inst::Phi, InstType},
        BBPtr, InstPtr, Operand, ValueType,
    },
    Program,
};

/// A single argument of "phi" instruction
type PhiArg = (BBPtr, Operand);

/// Pack of a "phi" instruction with corresponding variable
/// The variable is an "alloca" instruction
struct PhiPack {
    inst: InstPtr,
    variable: InstPtr,
}

impl PhiPack {
    /// Create a PhiPack from a variable
    /// The variable is the "alloca" instruction to be eliminated
    pub fn new_from_variable(variable: InstPtr, program: &mut Program, bb: &mut BBPtr) -> Self {
        // get type of phi variable
        let ValueType::Pointer(ty) = variable.get_value_type() else {
            panic!("variable type is not pointer");
        };

        // get and insert empty "phi" instruction
        let phi = program.mem_pool.get_phi(*ty, vec![]);
        bb.push_front(phi);

        // return phi pack
        Self {
            inst: phi,
            variable,
        }
    }

    /// Add an argument to the "phi" instruction
    pub fn add_argument(&mut self, phi_arg: PhiArg) {
        // get mutable reference of phi
        let phi = downcast_mut::<Phi>(self.inst.as_mut().as_mut());

        // add argument to phi
        phi.incoming_values.push((phi_arg.1, phi_arg.0));
    }
}

/// The mem2reg pass
#[allow(unused)]
pub fn mem2reg(entry: BBPtr, program: &mut Program) {
    let mut variable_to_phi_insertion: HashMap<InstPtr, Vec<BBPtr>> =
        get_variable_to_phi_insertion(entry);
    let mut block_to_phi_insertion: HashMap<BBPtr, Vec<PhiPack>> =
        insert_empty_phi(entry, program, &mut variable_to_phi_insertion);

    // for each "phi" insert position, decide the value for each argument
    fn decide_variable_value(
        variable: InstPtr,
        current_variable_value: &[HashMap<InstPtr, PhiArg>],
    ) -> (BBPtr, Operand) {
        for frame in current_variable_value.iter().rev() {
            if let Some(value) = frame.get(&variable) {
                return value.clone();
            }
        }
        panic!("variable value not found");
    }

    // start from entry node, decide the value for each "phi" instruction
    // this will also remove "load" and "store" instructions when possible
    fn decide_values_start_from(
        entry: BBPtr,
        visited: &mut HashSet<BBPtr>,
        current_variable_value: &mut Vec<HashMap<InstPtr, PhiArg>>,
        block_to_phi_insertion: &mut HashMap<BBPtr, Vec<PhiPack>>,
    ) {
        // decide value for each "phi" instruction to add
        for mut phi in block_to_phi_insertion
            .get_mut(&entry)
            .unwrap_or(&mut vec![])
            .iter_mut()
        {
            let new_phi_arg = decide_variable_value(phi.variable, current_variable_value);
            phi.add_argument(new_phi_arg);
            current_variable_value
                .last_mut()
                .unwrap()
                .insert(phi.variable, (entry, Operand::Instruction(phi.inst)));
        }

        // do not continue if visited
        // "phi" instruction can be added multiple times for each basic block
        // so it's put before this
        if visited.contains(&entry) {
            return;
        }
        visited.insert(entry);

        // iterate all instructions and:
        //
        // 1. for each "store", update current variable value and remove the "store"
        // 2. for each "load", replace with the current variable value
        //
        // bypass if featured variable is not a constant pointer,
        // for example if it's calculated from "getelementptr"
        //
        // does not remove "alloca" because it doesn't check if array is being accessed,
        // we use dead code elimination instead to remove unused "alloca"
        for mut inst in entry.iter() {
            match inst.get_type() {
                InstType::Store => {
                    let store_operands = inst.get_operand();
                    let store_ptr = &store_operands[1];
                    let store_value = &store_operands[0];

                    // update only when store destination is a constant pointer
                    if let Operand::Instruction(variable) = store_ptr {
                        if variable.get_type() == InstType::Alloca {
                            current_variable_value
                                .last_mut()
                                .unwrap()
                                .insert(*variable, (entry, store_value.clone()));
                            inst.remove_self();
                        }
                    }
                }
                InstType::Load => {
                    let load_operands = inst.get_operand();
                    let load_ptr = &load_operands[0];

                    // replace only when load source is a constant pointer
                    if let Operand::Instruction(variable) = load_ptr {
                        if variable.get_type() == InstType::Alloca {
                            let (_, new_value) =
                                decide_variable_value(*variable, current_variable_value);
                            inst.replace_self(&new_value);
                        }
                    }
                }
                _ => (),
            }
        }

        // visit all successors
        for succ in entry.get_succ_bb() {
            current_variable_value.push(HashMap::new());
            decide_values_start_from(
                *succ,
                visited,
                current_variable_value,
                block_to_phi_insertion,
            );
            current_variable_value.pop();
        }
    }

    // start mem2reg pass from the entry block
    decide_values_start_from(
        entry,
        &mut HashSet::new(),
        &mut vec![HashMap::new()],
        &mut block_to_phi_insertion,
    );
}

/// Insert empty "phi" for basic blocks starting from `entry`
/// Returns a mapping from basic block to inserted "phi" instructions
#[allow(unused)]
fn insert_empty_phi(
    entry: BBPtr,
    program: &mut Program,
    phi_insert_positions: &mut HashMap<InstPtr, Vec<BBPtr>>,
) -> HashMap<BBPtr, Vec<PhiPack>> {
    let mut block_to_phi_insertion: HashMap<BBPtr, Vec<PhiPack>> = HashMap::new();
    for (variable, positions) in phi_insert_positions.iter_mut() {
        for position in positions.iter_mut() {
            block_to_phi_insertion
                .entry(*position)
                .or_default()
                .push(PhiPack::new_from_variable(*variable, program, position));
        }
    }
    block_to_phi_insertion
}

/// Get places to insert "phi" instructions for each "alloca" instruction
#[allow(unused)]
fn get_variable_to_phi_insertion(entry: BBPtr) -> HashMap<InstPtr, Vec<BBPtr>> {
    let mut phi_positions = HashMap::new();

    // for each "store", make insert position at the beginning of the dominance frontier
    fn dfs_insertion(
        current_bb: BBPtr,
        visited: &mut HashSet<BBPtr>,
        df: &HashMap<BBPtr, HashSet<BBPtr>>,
        phi_positions: &mut HashMap<InstPtr, Vec<BBPtr>>,
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
                            phi_positions.entry(*inst).or_default().push(*df_bb);
                        }
                    }
                }
            }
        }
        for succ in current_bb.get_succ_bb() {
            dfs_insertion(*succ, visited, df, phi_positions);
        }
    }
    dfs_insertion(
        entry,
        &mut HashSet::new(),
        &get_dominance_frontiers(entry),
        &mut phi_positions,
    );

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
    use crate::{
        frontend::parse,
        middle::{ir::ValueType, irgen::gen, Program},
    };

    #[test]
    /// Optimize the following function:
    ///
    /// ```llvm
    /// define i32 @main {
    ///     %alloca_1 = alloca i32
    ///     store i32 1, ptr %alloca_1
    ///     %load_3 = load i32, ptr %alloca_1
    ///     ret i32 %load_3
    /// }
    /// ```
    ///
    /// To:
    ///
    /// ```llvm
    /// define i32 @main {
    ///     %alloca_1 = alloca i32
    ///     ret i32 1
    /// }
    /// ```
    ///
    /// `%alloca_1` is not removed because limitation of mem2reg pass (see code for details)
    /// It should be removed by dead code elimination pass instead
    fn test_mem2reg_simple() {
        let code = r#"
            int main() {
                int a = 1;
                return a;
            }
        "#;
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        assert_eq!(
            program.module.gen_llvm_ir(),
            "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 1, ptr %alloca_1\n%load_3 = load i32, ptr %alloca_1\nret %load_3\n\n\n}\n"
        );

        // check optimization
        mem2reg(program.module.functions[0].entry.unwrap(), &mut program);
        assert_eq!(
            program.module.gen_llvm_ir(),
            "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nret 1\n\n\n}\n"
        );
    }

    #[test]
    fn test_mem2reg_simple_if() {
        let code = r#"
            int main() {
                int x;
                if (1) {
                    x = 1;
                } else {
                    x = 2;
                }
                return x;
            }
        "#;
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        assert_eq!(
            program.module.gen_llvm_ir(),
            "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nbr label %cond0\n\n%cond0:\n%icmp_7 = icmp ne i32 1, 0\nbr i1 %icmp_7, label %then1, label %alt2\n\n%then1:\nstore i32 1, ptr %alloca_1\nbr label %final3\n\n%alt2:\nstore i32 2, ptr %alloca_1\nbr label %final3\n\n%final3:\n%load_13 = load i32, ptr %alloca_1\nret %load_13\n\n\n}\n"
        );

        // check optimization
        mem2reg(program.module.functions[0].entry.unwrap(), &mut program);
        assert_eq!(
            program.module.gen_llvm_ir(),
            "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nret 1\n\n\n}\n"
        );
    }

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
