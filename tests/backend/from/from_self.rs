mod test_gep_from_self {

    use compiler::{backend, middle};

    #[test]
    fn gen_gep_from_self() {
        // Create program
        let mut program = middle::Program::new();
        let mem_pool = &mut program.mem_pool;

        // Create basic block and fill instructions
        let mut entry = mem_pool.new_basicblock("entry".to_string());
        let value_type = middle::ir::ValueType::Array(
            middle::ir::ValueType::Array(
                (middle::ir::ValueType::Array(middle::ir::ValueType::Int.into(), 6)).into(),
                3,
            )
            .into(),
            4,
        );
        let alloca = mem_pool.get_alloca(value_type.clone(), 1);
        let gep = mem_pool.get_getelementptr(
            value_type,
            alloca.into(),
            vec![
                middle::ir::Operand::Constant(0.into()),
                middle::ir::Operand::Constant(1.into()),
                // middle::ir::Operand::Constant(2.into()),
            ],
        );
        let ret = mem_pool.get_ret(None);
        entry.push_back(alloca);
        entry.push_back(gep);
        entry.push_back(ret);

        // Wrap basic block in function and add to program
        let mut func = mem_pool.new_function("main".to_string(), middle::ir::ValueType::Void);
        func.entry = Some(entry);
        func.exit = Some(entry);
        program.module.functions.push(func);

        println!("{}", program.module.gen_llvm_ir());

        let program = backend::from_self::gen_from_self(&program).unwrap();

        let asm = program.gen_asm();

        println!("{}", asm);

        /* ---------- ---------- */
    }
}
