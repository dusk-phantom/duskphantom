use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    pin::Pin,
};

use anyhow::{Context, Result};

use crate::{
    context,
    middle::{
        ir::{
            instruction::{downcast_mut, misc_inst::Phi, InstType},
            BBPtr, IRBuilder, InstPtr, Operand, ValueType,
        },
        Program,
    },
};

#[allow(unused)]
pub fn optimize_program(program: &mut Program) -> Result<()> {
    for func in &program.module.functions {
        if let Some(entry) = func.entry {
            mem2reg(entry, &mut program.mem_pool)?;
        }
    }
    Ok(())
}

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
    /// Errors when variable is not of pointer type
    pub fn new_from_variable(
        variable: InstPtr,
        mem_pool: &mut Pin<Box<IRBuilder>>,
        bb: &mut BBPtr,
    ) -> Result<Self> {
        // Get type of phi variable
        let ValueType::Pointer(ty) = variable.get_value_type() else {
            return Err(anyhow::anyhow!("variable type is not pointer"))
                .with_context(|| context!());
        };

        // Get and insert empty "phi" instruction
        let phi = mem_pool.get_phi(*ty, vec![]);
        bb.push_front(phi);

        // Return phi pack
        Ok(Self {
            inst: phi,
            variable,
        })
    }

    /// Add an argument to the "phi" instruction
    pub fn add_argument(&mut self, phi_arg: PhiArg) {
        // Get mutable reference of phi
        let phi = downcast_mut::<Phi>(self.inst.as_mut().as_mut());

        // Add argument to phi
        phi.incoming_values.push((phi_arg.1, phi_arg.0));
    }
}

/// The mem2reg pass
#[allow(unused)]
pub fn mem2reg(entry: BBPtr, mem_pool: &mut Pin<Box<IRBuilder>>) -> Result<()> {
    let mut variable_to_phi_insertion: BTreeMap<InstPtr, BTreeSet<BBPtr>> =
        get_variable_to_phi_insertion(entry);
    let mut block_to_phi_insertion: BTreeMap<BBPtr, Vec<PhiPack>> =
        insert_empty_phi(entry, mem_pool, variable_to_phi_insertion)?;

    /// For each "phi" insert position, decide the value for each argument
    /// Errors when variable is not found in current_variable_value
    fn decide_variable_value(
        variable: InstPtr,
        current_variable_value: &[BTreeMap<InstPtr, Operand>],
    ) -> Result<Operand> {
        for frame in current_variable_value.iter().rev() {
            if let Some(value) = frame.get(&variable) {
                return Ok(value.clone());
            }
        }
        let ValueType::Pointer(ptr) = variable.get_value_type() else {
            return Err(anyhow::anyhow!("variable type is not pointer"))
                .with_context(|| context!());
        };

        // Value not found can happen when out of scope of a variable, or not defined
        // To keep consistent with LLVM, return default initializer
        Ok(Operand::Constant(ptr.default_initializer()?))
    }

    /// Start from entry node, decide the value for each "phi" instruction
    /// This will also remove "load" and "store" instructions when possible
    fn decide_values_start_from(
        parent_bb: Option<BBPtr>,
        current_bb: BBPtr,
        visited: &mut BTreeSet<BBPtr>,
        current_variable_value: &mut Vec<BTreeMap<InstPtr, Operand>>,
        block_to_phi_insertion: &mut BTreeMap<BBPtr, Vec<PhiPack>>,
    ) -> Result<()> {
        // Decide value for each "phi" instruction to add
        for mut phi in block_to_phi_insertion
            .get_mut(&current_bb)
            .unwrap_or(&mut vec![])
            .iter_mut()
        {
            let value = decide_variable_value(phi.variable, current_variable_value)?;
            phi.add_argument((parent_bb.unwrap(), value));
            current_variable_value
                .last_mut()
                .unwrap()
                .insert(phi.variable, Operand::Instruction(phi.inst));
        }

        // Do not continue if visited
        // "phi" instruction can be added multiple times for each basic block,
        // so that part is before this check
        if visited.contains(&current_bb) {
            return Ok(());
        }
        visited.insert(current_bb);

        // Iterate all instructions and:
        //
        // 1. for each "store", update current variable value and remove the "store"
        // 2. for each "load", replace with the current variable value
        //
        // Bypass if featured variable is not a constant pointer,
        // for example if it's calculated from "getelementptr"
        for mut inst in current_bb.iter() {
            match inst.get_type() {
                InstType::Store => {
                    let store_operands = inst.get_operand();
                    let store_ptr = &store_operands[1];
                    let store_value = &store_operands[0];

                    // Update only when store destination is a constant pointer
                    if let Operand::Instruction(variable) = store_ptr {
                        if variable.get_type() == InstType::Alloca {
                            current_variable_value
                                .last_mut()
                                .unwrap()
                                .insert(*variable, store_value.clone());
                            inst.remove_self();
                        }
                    }
                }
                InstType::Load => {
                    let load_operands = inst.get_operand();
                    let load_ptr = &load_operands[0];

                    // Replace only when load source is a constant pointer
                    if let Operand::Instruction(variable) = load_ptr {
                        if variable.get_type() == InstType::Alloca {
                            let current_value =
                                decide_variable_value(*variable, current_variable_value)?;
                            inst.replace_self(&current_value);
                        }
                    }
                }
                _ => (),
            }
        }

        // Visit all successors
        let successors = current_bb.get_succ_bb();
        let need_new_frame = successors.len() > 1;
        for succ in successors {
            // Only add new frame if there is more than one successors
            if need_new_frame {
                current_variable_value.push(BTreeMap::new());
            }
            decide_values_start_from(
                Some(current_bb),
                *succ,
                visited,
                current_variable_value,
                block_to_phi_insertion,
            )?;
            if need_new_frame {
                current_variable_value.pop();
            }
        }
        Ok(())
    }

    // Start mem2reg pass from the entry block
    decide_values_start_from(
        None,
        entry,
        &mut BTreeSet::new(),
        &mut vec![BTreeMap::new()],
        &mut block_to_phi_insertion,
    )
}

/// Insert empty "phi" for basic blocks starting from `entry`
/// Returns a mapping from basic block to inserted "phi" instructions
#[allow(unused)]
fn insert_empty_phi(
    entry: BBPtr,
    mem_pool: &mut Pin<Box<IRBuilder>>,
    phi_insert_positions: BTreeMap<InstPtr, BTreeSet<BBPtr>>,
) -> Result<BTreeMap<BBPtr, Vec<PhiPack>>> {
    let mut block_to_phi_insertion: BTreeMap<BBPtr, Vec<PhiPack>> = BTreeMap::new();
    for (variable, positions) in phi_insert_positions.into_iter() {
        for mut position in positions.into_iter() {
            block_to_phi_insertion
                .entry(position)
                .or_default()
                .push(PhiPack::new_from_variable(
                    variable,
                    mem_pool,
                    &mut position,
                )?);
        }
    }
    Ok(block_to_phi_insertion)
}

/// Get places to insert "phi" instructions for each "alloca" instruction
#[allow(unused)]
fn get_variable_to_phi_insertion(entry: BBPtr) -> BTreeMap<InstPtr, BTreeSet<BBPtr>> {
    let mut phi_positions: BTreeMap<InstPtr, BTreeSet<BBPtr>> = BTreeMap::new();
    let mut store_positions: BTreeMap<InstPtr, BTreeSet<BBPtr>> = BTreeMap::new();
    let dominance_frontiers: BTreeMap<BBPtr, BTreeSet<BBPtr>> = get_dominance_frontiers(entry);

    /// Build a mapping from variable to store positions
    fn build_store_positions(
        current_bb: BBPtr,
        visited: &mut HashSet<BBPtr>,
        store_positions: &mut BTreeMap<InstPtr, BTreeSet<BBPtr>>,
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

                // Only insert "phi" when store destination is a constant pointer
                if let Operand::Instruction(inst) = store_ptr {
                    if inst.get_type() == InstType::Alloca {
                        store_positions.entry(*inst).or_default().insert(current_bb);
                    }
                }
            }
        }
        for succ in current_bb.get_succ_bb() {
            build_store_positions(*succ, visited, store_positions);
        }
    }

    // For each variable, find all dominance frontiers and insert "phi" instructions
    // After inserting "phi" at a block, find its dominance frontiers and insert "phi" recursively
    build_store_positions(entry, &mut HashSet::new(), &mut store_positions);
    for (variable, vis) in store_positions.iter_mut() {
        let mut positions = vis.clone();
        while !positions.is_empty() {
            let position = positions.pop_first().unwrap();
            let Some(dfs) = dominance_frontiers.get(&position) else {
                continue;
            };
            for df in dfs {
                phi_positions.entry(*variable).or_default().insert(*df);

                // Only insert positions never considered before
                if (!vis.contains(df)) {
                    vis.insert(*df);
                    positions.insert(*df);
                }
            }
        }
    }

    // Return result
    phi_positions
}

/// Get dominance frontiers of each basic block in the function
#[allow(unused)]
fn get_dominance_frontiers(entry: BBPtr) -> BTreeMap<BBPtr, BTreeSet<BBPtr>> {
    let idoms = get_immediate_dominators(entry);
    let mut df = BTreeMap::new();

    /// Calculate dominance frontiers
    for (bb, idom) in idoms.iter() {
        if bb == idom {
            continue;
        }
        for pred in bb.get_pred_bb() {
            let mut runner = *pred;
            while runner != idoms[bb] {
                df.entry(runner).or_insert(BTreeSet::new()).insert(*bb);

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
fn get_immediate_dominators(entry: BBPtr) -> BTreeMap<BBPtr, BBPtr> {
    let mut idoms = BTreeMap::new();
    idoms.insert(entry, entry);

    /// Calculate postorder with dfs
    fn dfs_postorder(
        current_bb: BBPtr,
        visited: &mut BTreeSet<BBPtr>,
        postorder_map: &mut BTreeMap<BBPtr, i32>,
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
    let mut postorder_map = BTreeMap::new();
    let mut postorder_array = Vec::new();
    dfs_postorder(
        entry,
        &mut BTreeSet::new(),
        &mut postorder_map,
        &mut postorder_array,
    );

    /// Function to get lowest common ancestor of two basic blocks in the dominator tree
    fn intersect(
        mut n: BBPtr,
        mut m: BBPtr,
        postorder_map: &BTreeMap<BBPtr, i32>,
        idoms: &BTreeMap<BBPtr, BBPtr>,
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
pub mod tests_mem2reg {
    use super::*;
    use crate::middle::{ir::ValueType, Program};

    #[test]
    fn test_get_idoms() {
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
        let idoms = get_immediate_dominators(entry);

        // Check if idoms are correct
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
        let df = get_dominance_frontiers(entry);

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

    #[test]
    fn test_phi_insert_positions_single() {
        let mut program = Program::new();

        // Construct a function with "alloca" and "store" instructions
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

        // Calculate df and phi insert positions
        let phi_positions = get_variable_to_phi_insertion(entry);

        // Check if phi insert positions are correct
        assert_eq!(phi_positions.len(), 0);
    }

    #[test]
    fn test_phi_insert_positions_nested() {
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

        // Construct "alloca" and "store" instructions
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

        // Calculate phi insert positions
        let phi_positions = get_variable_to_phi_insertion(entry);

        // Check if phi insert positions are correct
        assert_eq!(phi_positions.len(), 3);
        assert_eq!(phi_positions[&alloca1].len(), 1);
        assert_eq!(phi_positions[&alloca2].len(), 1);
        assert_eq!(phi_positions[&alloca3].len(), 1);
        assert!(phi_positions[&alloca1].contains(&end));
        assert!(phi_positions[&alloca2].contains(&end));
        assert!(phi_positions[&alloca3].contains(&end));
    }
}
