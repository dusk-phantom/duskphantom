use compiler::{backend::irs::*, utils::diff::diff};
use insta::{assert_debug_snapshot, assert_snapshot};

#[test]
fn test_handle_offset_overflow() {
    let mut entry = Block::new("".to_string());
    let mut ssa = StackAllocator::new();
    let mut rg = RegGenerator::new();
    [(); 300].into_iter().for_each(|_| {
        ssa.alloc(8);
    });
    let ss = ssa.alloc(8);
    let dst = rg.gen_virtual_usual_reg();
    let ld = LoadInst::new(dst, ss).into();
    entry.push_inst(ld);
    entry.push_inst(Inst::Ret);
    let mut f = Func::new("test".to_string(), vec![], entry);
    f.stack_allocator_mut().replace(ssa);
    f.reg_gener_mut().replace(rg);

    phisicalize_func(&mut f).unwrap();

    assert_debug_snapshot!(f,@r###"
    Func {
        name: "test",
        args: [],
        ret: None,
        reg_gener: Some(
            RegGenerator {
                usual_counter: ParalCounter {
                    end: 100000000,
                    counter: 33,
                },
                float_counter: ParalCounter {
                    end: 100000000,
                    counter: 32,
                },
            },
        ),
        stack_allocator: Some(
            StackAllocator {
                alloc_from: 2424,
            },
        ),
        entry: Block {
            label: "",
            insts: [
                Sd(
                    SdInst(
                        Reg {
                            id: 8,
                            is_usual: true,
                        },
                        Imm(
                            -8,
                        ),
                        Reg {
                            id: 2,
                            is_usual: true,
                        },
                    ),
                ),
                Mv(
                    MvInst(
                        Reg(
                            Reg {
                                id: 8,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 2,
                                is_usual: true,
                            },
                        ),
                    ),
                ),
                Li(
                    LiInst(
                        Reg(
                            Reg {
                                id: 5,
                                is_usual: true,
                            },
                        ),
                        Imm(
                            Imm(
                                -2432,
                            ),
                        ),
                    ),
                ),
                Add(
                    AddInst(
                        Reg(
                            Reg {
                                id: 2,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 2,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 5,
                                is_usual: true,
                            },
                        ),
                        true,
                    ),
                ),
                Li(
                    LiInst(
                        Reg(
                            Reg {
                                id: 28,
                                is_usual: true,
                            },
                        ),
                        Imm(
                            Imm(
                                2400,
                            ),
                        ),
                    ),
                ),
                Add(
                    AddInst(
                        Reg(
                            Reg {
                                id: 28,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 28,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 2,
                                is_usual: true,
                            },
                        ),
                        true,
                    ),
                ),
                Lw(
                    LwInst(
                        Reg {
                            id: 5,
                            is_usual: true,
                        },
                        Imm(
                            0,
                        ),
                        Reg {
                            id: 28,
                            is_usual: true,
                        },
                    ),
                ),
                Li(
                    LiInst(
                        Reg(
                            Reg {
                                id: 28,
                                is_usual: true,
                            },
                        ),
                        Imm(
                            Imm(
                                2408,
                            ),
                        ),
                    ),
                ),
                Add(
                    AddInst(
                        Reg(
                            Reg {
                                id: 28,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 28,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 2,
                                is_usual: true,
                            },
                        ),
                        true,
                    ),
                ),
                Sd(
                    SdInst(
                        Reg {
                            id: 5,
                            is_usual: true,
                        },
                        Imm(
                            0,
                        ),
                        Reg {
                            id: 28,
                            is_usual: true,
                        },
                    ),
                ),
                Mv(
                    MvInst(
                        Reg(
                            Reg {
                                id: 2,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 8,
                                is_usual: true,
                            },
                        ),
                    ),
                ),
                Ld(
                    LdInst(
                        Reg {
                            id: 8,
                            is_usual: true,
                        },
                        Imm(
                            -8,
                        ),
                        Reg {
                            id: 8,
                            is_usual: true,
                        },
                    ),
                ),
                Ret,
            ],
        },
        other_bbs: [],
    }
    "###);
}

#[test]
fn test_handle_long_jmp_for_jmp() {
    use compiler::backend::irs::*;
    let mut bb0 = Block::new("bb0".to_string());
    let mut bb1 = Block::new("bb1".to_string());
    let mut bb2 = Block::new("bb2".to_string());
    bb0.push_inst(JmpInst::new("bb2".into()).into());
    [(); 2]
        .iter()
        .for_each(|_| bb1.push_inst(LiInst::new(REG_A0.into(), 1.into()).into()));
    bb1.push_inst(Inst::Ret);
    bb2.push_inst(JmpInst::new("bb1".into()).into());
    let mut f = Func::new("main".to_string(), vec![], bb0);
    f.push_bb(bb1);
    f.push_bb(bb2);
    let f_asm_before = f.gen_asm();
    handle_long_jump(&mut f, &REG_A1, 3).unwrap();
    let f_asm_after = f.gen_asm();
    assert_snapshot!(diff(&f_asm_before,& f_asm_after),@r###"
    .text
    .align	3
    .globl	main
    .type	main, @function
    main:
    bb0:
    j bb2
    bb1:
    li a0,1
    li a0,1
    ret
    bb2:
    [-] j bb1
    [+] lla a1,bb1
    [+] jalr zero,a1,0
    .size	main, .-main
    "###);
}

#[test]
fn test_handle_long_jmp_for_branch() {
    let mut bb0 = Block::new("bb0".to_string());
    let mut bb1 = Block::new("bb1".to_string());
    let mut bb2 = Block::new("bb2".to_string());
    bb0.push_inst(LiInst::new(REG_A0.into(), 1.into()).into());
    bb0.push_inst(LiInst::new(REG_A1.into(), 1.into()).into());
    bb0.push_inst(BeqInst::new(REG_A0, REG_A1, "bb2".into()).into());
    bb0.push_inst(JmpInst::new("bb1".into()).into());
    bb1.push_inst(JmpInst::new("bb2".into()).into());
    bb2.push_inst(LiInst::new(REG_A0.into(), 2.into()).into());
    bb2.push_inst(Inst::Ret);
    [(); 10]
        .iter()
        .for_each(|_| bb1.push_inst(LiInst::new(REG_A0.into(), 1.into()).into()));
    let mut f = Func::new("main".to_string(), vec![], bb0);
    f.push_bb(bb1);
    f.push_bb(bb2);
    let f_asm_before = f.gen_asm();
    handle_long_jump(&mut f, &REG_A2, 10).unwrap();
    let f_asm_after = f.gen_asm();
    assert_snapshot!(diff(&f_asm_before,& f_asm_after),@r###"
    .text
    .align	3
    .globl	main
    .type	main, @function
    main:
    bb0:
    li a0,1
    li a1,1
    [-] beq a0,a1,bb2
    [+] beq a0,a1,main_long_jmp_1
    j bb1
    [+] main_long_jmp_1:
    [+] lla a2,bb2
    [+] jalr zero,a2,0
    bb1:
    j bb2
    li a0,1
    li a0,1
    li a0,1
    li a0,1
    li a0,1
    li a0,1
    li a0,1
    li a0,1
    li a0,1
    li a0,1
    bb2:
    li a0,2
    ret
    .size	main, .-main
    "###);
}
