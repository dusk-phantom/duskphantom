use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result};

use crate::{
    context,
    middle::{
        ir::{
            instruction::{downcast_mut, misc_inst::Phi, InstType},
            BBPtr, InstPtr, Operand, ValueType,
        },
        Program,
    },
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
    /// Errors when variable is not of pointer type
    pub fn new_from_variable(
        variable: InstPtr,
        program: &mut Program,
        bb: &mut BBPtr,
    ) -> Result<Self> {
        // Get type of phi variable
        let ValueType::Pointer(ty) = variable.get_value_type() else {
            return Err(anyhow::anyhow!("variable type is not pointer"))
                .with_context(|| context!());
        };

        // Get and insert empty "phi" instruction
        let phi = program.mem_pool.get_phi(*ty, vec![]);
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
pub fn mem2reg(entry: BBPtr, program: &mut Program) -> Result<()> {
    let mut variable_to_phi_insertion: BTreeMap<InstPtr, BTreeSet<BBPtr>> =
        get_variable_to_phi_insertion(entry);
    let mut block_to_phi_insertion: BTreeMap<BBPtr, Vec<PhiPack>> =
        insert_empty_phi(entry, program, variable_to_phi_insertion)?;

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
        Err(anyhow::anyhow!("variable value not found")).with_context(|| context!())
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
    program: &mut Program,
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
                    program,
                    &mut position,
                )?);
        }
    }
    Ok(block_to_phi_insertion)
}

/// Get places to insert "phi" instructions for each "alloca" instruction
#[allow(unused)]
fn get_variable_to_phi_insertion(entry: BBPtr) -> BTreeMap<InstPtr, BTreeSet<BBPtr>> {
    let mut phi_positions = BTreeMap::new();

    /// For each "store", make insert position at the beginning of the dominance frontier
    fn dfs_insertion(
        current_bb: BBPtr,
        visited: &mut BTreeSet<BBPtr>,
        df: &BTreeMap<BBPtr, BTreeSet<BBPtr>>,
        phi_positions: &mut BTreeMap<InstPtr, BTreeSet<BBPtr>>,
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
                        for df_bb in df.get(&current_bb).unwrap_or(&BTreeSet::new()).iter() {
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
    dfs_insertion(
        entry,
        &mut BTreeSet::new(),
        &get_dominance_frontiers(entry),
        &mut phi_positions,
    );

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
                runner = idoms[&runner];
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
    use insta::assert_snapshot;

    use super::*;
    use crate::{
        frontend::parse,
        middle::{ir::ValueType, irgen::gen, Program},
    };

    #[test]
    fn test_mem2reg_simple() {
        let code = r#"
            int main() {
                int a = 1;
                return a;
            }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        mem2reg(program.module.functions[0].entry.unwrap(), &mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(format!(
            "BEFORE:\n{}\n\nAFTER:\n{}",
            llvm_before, llvm_after
        ), @r###"
        BEFORE:
        define i32 @main() {
        entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        store i32 1, ptr %alloca_5
        %load_7 = load i32, ptr %alloca_5
        store i32 %load_7, ptr %alloca_2
        br label %exit

        exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }


        AFTER:
        define i32 @main() {
        entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        br label %exit

        exit:
        ret i32 1


        }
        "###);
    }

    #[test]
    fn test_mem2reg_branch() {
        let code = r#"
            int main() {
                int x = 0;
                if (x < 10) {
                    x = x + 1;
                } else {
                    x = x + 9;
                }
                return x;
            }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        mem2reg(program.module.functions[0].entry.unwrap(), &mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(format!(
            "BEFORE:\n{}\n\nAFTER:\n{}",
            llvm_before, llvm_after
        ), @r###"
        BEFORE:
        define i32 @main() {
        entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        store i32 0, ptr %alloca_5
        br label %cond0

        cond0:
        %load_12 = load i32, ptr %alloca_5
        %icmp_13 = icmp slt i32 %load_12, 10
        br i1 %icmp_13, label %then1, label %alt2

        then1:
        %load_15 = load i32, ptr %alloca_5
        %Add_16 = add i32 %load_15, 1
        store i32 %Add_16, ptr %alloca_5
        br label %final3

        alt2:
        %load_19 = load i32, ptr %alloca_5
        %Add_20 = add i32 %load_19, 9
        store i32 %Add_20, ptr %alloca_5
        br label %final3

        final3:
        %load_23 = load i32, ptr %alloca_5
        store i32 %load_23, ptr %alloca_2
        br label %exit

        exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }


        AFTER:
        define i32 @main() {
        entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        br label %cond0

        cond0:
        %icmp_13 = icmp slt i32 0, 10
        br i1 %icmp_13, label %then1, label %alt2

        then1:
        %Add_16 = add i32 0, 1
        br label %final3

        alt2:
        %Add_20 = add i32 0, 9
        br label %final3

        final3:
        %phi_26 = phi i32 [%Add_16, %then1], [%Add_20, %alt2]
        br label %exit

        exit:
        ret i32 %phi_26


        }
        "###);
    }

    #[test]
    fn test_mem2reg_loop() {
        let code = r#"
            int main() {
                int x = 0;
                while (x < 10) {
                    x = x + 1;
                }
                return x;
            }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        mem2reg(program.module.functions[0].entry.unwrap(), &mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(format!(
            "BEFORE:\n{}\n\nAFTER:\n{}",
            llvm_before, llvm_after
        ), @r###"
        BEFORE:
        define i32 @main() {
        entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        store i32 0, ptr %alloca_5
        br label %cond0

        cond0:
        %load_15 = load i32, ptr %alloca_5
        %icmp_16 = icmp slt i32 %load_15, 10
        br i1 %icmp_16, label %body1, label %final2

        body1:
        %load_11 = load i32, ptr %alloca_5
        %Add_12 = add i32 %load_11, 1
        store i32 %Add_12, ptr %alloca_5
        br label %cond0

        final2:
        %load_18 = load i32, ptr %alloca_5
        store i32 %load_18, ptr %alloca_2
        br label %exit

        exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }


        AFTER:
        define i32 @main() {
        entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        br label %cond0

        cond0:
        %phi_21 = phi i32 [0, %entry], [%Add_12, %body1]
        %icmp_16 = icmp slt i32 %phi_21, 10
        br i1 %icmp_16, label %body1, label %final2

        body1:
        %Add_12 = add i32 %phi_21, 1
        br label %cond0

        final2:
        br label %exit

        exit:
        ret i32 %phi_21


        }
        "###);
    }

    #[test]
    fn test_mem2reg_nested() {
        let code = r#"
            int main() {
                int x = 0;
                while (x < 10) {
                    x = x + 2;
                    if (x > 5) while (x < 8) x = x + 1;
                }
                return x;
            }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        mem2reg(program.module.functions[0].entry.unwrap(), &mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(format!(
            "BEFORE:\n{}\n\nAFTER:\n{}",
            llvm_before, llvm_after
        ), @r###"
        BEFORE:
        define i32 @main() {
        entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        store i32 0, ptr %alloca_5
        br label %cond0

        cond0:
        %load_36 = load i32, ptr %alloca_5
        %icmp_37 = icmp slt i32 %load_36, 10
        br i1 %icmp_37, label %body1, label %final2

        body1:
        %load_11 = load i32, ptr %alloca_5
        %Add_12 = add i32 %load_11, 2
        store i32 %Add_12, ptr %alloca_5
        br label %cond3

        final2:
        %load_39 = load i32, ptr %alloca_5
        store i32 %load_39, ptr %alloca_2
        br label %exit

        cond3:
        %load_19 = load i32, ptr %alloca_5
        %icmp_20 = icmp sgt i32 %load_19, 5
        br i1 %icmp_20, label %then4, label %alt5

        exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3

        then4:
        br label %cond7

        alt5:
        br label %final6

        cond7:
        %load_30 = load i32, ptr %alloca_5
        %icmp_31 = icmp slt i32 %load_30, 8
        br i1 %icmp_31, label %body8, label %final9

        final6:
        br label %cond0

        body8:
        %load_26 = load i32, ptr %alloca_5
        %Add_27 = add i32 %load_26, 1
        store i32 %Add_27, ptr %alloca_5
        br label %cond7

        final9:
        br label %final6


        }


        AFTER:
        define i32 @main() {
        entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        br label %cond0

        cond0:
        %phi_42 = phi i32 [0, %entry], [%phi_43, %final6]
        %icmp_37 = icmp slt i32 %phi_42, 10
        br i1 %icmp_37, label %body1, label %final2

        body1:
        %Add_12 = add i32 %phi_42, 2
        br label %cond3

        final2:
        br label %exit

        cond3:
        %icmp_20 = icmp sgt i32 %Add_12, 5
        br i1 %icmp_20, label %then4, label %alt5

        exit:
        ret i32 %phi_42

        then4:
        br label %cond7

        alt5:
        br label %final6

        cond7:
        %phi_43 = phi i32 [%Add_12, %then4], [%Add_27, %body8]
        %icmp_31 = icmp slt i32 %phi_43, 8
        br i1 %icmp_31, label %body8, label %final9

        final6:
        br label %cond0

        body8:
        %Add_27 = add i32 %phi_43, 1
        br label %cond7

        final9:
        br label %final6


        }
        "###);
    }

    #[test]
    fn test_mem2reg_array() {
        let code = r#"
            int main() {
                int arr[1] = {8};
                f(arr);
                putarray(1, arr);
                return 0;
            }

            int f(int a[]) {
                a[0] = 1;
                return a[0];
            }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        mem2reg(program.module.functions[0].entry.unwrap(), &mut program).unwrap();
        mem2reg(program.module.functions[1].entry.unwrap(), &mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(format!(
            "BEFORE:\n{}\n\nAFTER:\n{}",
            llvm_before, llvm_after
        ), @r###"
        BEFORE:
        define i32 @main() {
        entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca [1 x i32]
        %getelementptr_6 = getelementptr [1 x i32], ptr %alloca_5, i32 0, i32 0
        %getelementptr_7 = getelementptr i32, ptr %getelementptr_6, i32 0
        store i32 8, ptr %getelementptr_7
        %getelementptr_9 = getelementptr [1 x i32], ptr %alloca_5, i32 0, i32 0
        %call_10 = call i32 @f(i32* %getelementptr_9)
        %getelementptr_11 = getelementptr [1 x i32], ptr %alloca_5, i32 0, i32 0
        %call_12 = call void @putarray(i32 1, i32* %getelementptr_11)
        store i32 0, ptr %alloca_2
        br label %exit

        exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        define i32 @f(i32* %a) {
        entry:
        %alloca_17 = alloca i32
        %alloca_20 = alloca i32*
        store i32* %a, ptr %alloca_20
        %load_22 = load i32*, ptr %alloca_20
        store i32 1, ptr %load_22
        %load_24 = load i32*, ptr %alloca_20
        %load_25 = load i32, ptr %load_24
        store i32 %load_25, ptr %alloca_17
        br label %exit

        exit:
        %load_18 = load i32, ptr %alloca_17
        ret i32 %load_18


        }


        AFTER:
        define i32 @main() {
        entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca [1 x i32]
        %getelementptr_6 = getelementptr [1 x i32], ptr %alloca_5, i32 0, i32 0
        %getelementptr_7 = getelementptr i32, ptr %getelementptr_6, i32 0
        store i32 8, ptr %getelementptr_7
        %getelementptr_9 = getelementptr [1 x i32], ptr %alloca_5, i32 0, i32 0
        %call_10 = call i32 @f(i32* %getelementptr_9)
        %getelementptr_11 = getelementptr [1 x i32], ptr %alloca_5, i32 0, i32 0
        %call_12 = call void @putarray(i32 1, i32* %getelementptr_11)
        br label %exit

        exit:
        ret i32 0


        }
        define i32 @f(i32* %a) {
        entry:
        %alloca_17 = alloca i32
        %alloca_20 = alloca i32*
        store i32 1, ptr %a
        %load_25 = load i32, ptr %a
        br label %exit

        exit:
        ret i32 %load_25


        }
        "###);
    }

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
