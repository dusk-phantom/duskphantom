#[cfg(test)]
pub mod tests_mem2reg {
    use insta::assert_snapshot;

    use compiler::{
        frontend::parse,
        middle::{
            irgen::gen,
            transform::{deadcode_elimination, mem2reg},
        },
        utils::diff::diff,
    };

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
        assert_snapshot!(diff(&llvm_before, &llvm_after), @r###"
        @loopCount = dso_local global i32 0
        @global = dso_local global i32 0
        declare i32 @getint()
        declare i32 @getch()
        declare float @getfloat()
        declare void @putint(i32 %p0)
        declare void @putch(i32 %p0)
        declare void @putfloat(float %p0)
        declare i32 @getarray(i32* %p0)
        declare i32 @getfarray(float* %p0)
        declare void @putarray(i32 %p0, i32* %p1)
        declare void @putfarray(i32 %p0, float* %p1)
        declare void @_sysy_starttime(i32 %p0)
        declare void @_sysy_stoptime(i32 %p0)
        declare void @putf()
        declare void @llvm.memset.p0.i32(i32* %p0, i8 %p1, i32 %p2, i1 %p3)
        define void @func(i32 %i0) {
        entry:
        [-] %alloca_3 = alloca i32
        [-] %alloca_5 = alloca i32
        [-] %alloca_7 = alloca i32
        [-] %alloca_9 = alloca i32
        [-] %alloca_11 = alloca i32
        [-] %alloca_13 = alloca i32
        store i32 %i0, ptr @global
        br label %exit

        exit:
        ret void


        }
        define i32 @main() {
        entry:
        [-] %alloca_20 = alloca i32
        [-] %alloca_23 = alloca i32
        [-] %alloca_25 = alloca i32
        %call_27 = call i32 @getint()
        store i32 %call_27, ptr @loopCount
        call void @_sysy_starttime(i32 21)
        br label %cond0

        cond0:
        [-] %phi_83 = phi i32 [0, %entry], [%phi_84, %final5]
        [-] %phi_81 = phi i32 [0, %entry], [%SDiv_56, %final5]
        [-] %phi_80 = phi i32 [0, %entry], [%Add_66, %final5]
        [-] %phi_79 = phi i32 [0, %entry], [%SRem_63, %final5]
        %load_70 = load i32, ptr @loopCount
        %icmp_71 = icmp slt i32 %phi_80, %load_70
        br i1 %icmp_71, label %body1, label %final2

        body1:
        [-] %alloca_34 = alloca i32
        [-] %alloca_36 = alloca i32
        br label %cond3

        final2:
        call void @_sysy_stoptime(i32 37)
        call void @putint(i32 %phi_79)
        call void @putch(i32 10)
        br label %exit

        cond3:
        [-] %phi_84 = phi i32 [0, %body1], [%Add_49, %body4]
        [-] %phi_82 = phi i32 [0, %body1], [%Add_46, %body4]
        %icmp_53 = icmp slt i32 %phi_84, 60
        br i1 %icmp_53, label %body4, label %final5

        exit:
        ret i32 0

        body4:
        call void @func(i32 %phi_80)
        [-] %load_45 = load i32, ptr @global
        [-] %Add_46 = add i32 %phi_82, %load_45
        [-] %Add_49 = add i32 %phi_84, 1
        br label %cond3

        final5:
        [-] %SDiv_56 = sdiv i32 %phi_82, 60
        [-] %Add_60 = add i32 %phi_79, %SDiv_56
        [-] %SRem_63 = srem i32 %Add_60, 134209537
        [-] %Add_66 = add i32 %phi_80, 1
        br label %cond0


        }
        "###);
    }
}
