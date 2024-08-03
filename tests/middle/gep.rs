#[cfg(test)]
pub mod test_gep {
    use compiler::{backend::Operand, middle::ir::*};
    #[test]
    fn gep_example() {
        let mut ir_builder = IRBuilder::new();
        let mut module = Module::new(ir_builder);

        let mut func = ir_builder.new_function("test_gep".to_owned(), ValueType::Void);
        module.functions.push(func);

        let mut bb = ir_builder.new_basicblock("test_gep_bb".to_owned());
        func.entry = Some(bb);
        func.exit = Some(bb);

        bb.push_back(ir_builder.get_ret(None));
        let tail = bb.get_last_inst();

        let a_type = ValueType::Array(
            ValueType::Array(ValueType::Array(ValueType::Int, 10), 10),
            10,
        );

        // int a[10][10][10];
        let a = ir_builder.get_alloca(
            ValueType::Array(ValueType::Array(ValueType::Int, 10), 10),
            10,
        );

        // a[1][2][3] = 4;
        let gep = ir_builder.get_getelementptr(
            a_type,
            a,
            vec![
                Operand::Constant(0),
                Operand::Constant(1),
                Operand::Constant(2),
                Operand::Constant(3),
            ],
        );

        let st = ir_builder.get_store(operand::Operand::Constant(4), gep);

        tail.insert_before(a);
        tail.insert_before(gep);
        tail.insert_before(st);
        assert!(true);
    }
}
