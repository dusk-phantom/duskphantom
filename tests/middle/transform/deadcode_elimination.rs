#[cfg(test)]
pub mod tests_mem2reg {
    use insta::assert_snapshot;

    use compiler::{
        frontend::parse,
        middle::{
            irgen::gen,
            transform::{deadcode_elimination, mem2reg},
        },
    };
    use prettydiff::diff_lines;

    #[test]
    fn test_dce_1() {
        let code = r#"
        int loopCount = 0;
        int global = 0;

        void func(int i0)
        {
            int i1 = 1;
            int i2 = 2;
            int i3 = 3;
            int i4 = 4;
            int i5 = 5;
            global = i0;
            return;
        }

        int main()
        {
            int sum = 0;
            int i = 0;
            loopCount = getint();
            starttime();
            while(i<loopCount)
            {
                int tmp = 0;
                int j = 0;
                while(j<60)
                {
                func(i);
                tmp = tmp + global;
                j = j + 1;
                }
                tmp = tmp / 60;
                sum = sum + tmp;
                sum = sum % 134209537;
                i = i + 1;
            }
            stoptime();
            putint(sum);
            putch(10);
            return 0;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        deadcode_elimination::optimize_program(&mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff_lines(&llvm_before, &llvm_after), @"");
    }
}
