use compiler::backend::irs::*;
use insta::assert_debug_snapshot;

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
            to_bbs: [],
        },
        other_bbs: [],
    }
    "###);
}

#[test]
fn test_mul_div_opt() {
    let mut entry = Block::new("entry".to_string());
    let ssa = StackAllocator::new();
    let mut rg = RegGenerator::new();

    // li dst, 0
    let dst = rg.gen_virtual_usual_reg();
    let lhs = rg.gen_virtual_usual_reg();
    let mul = MulInst::new(dst.into(), lhs.into(), 0.into());
    entry.push_inst(mul.into());

    // left shift 5
    let dst = rg.gen_virtual_usual_reg();
    let lhs = rg.gen_virtual_usual_reg();
    let mul = MulInst::new(dst.into(), lhs.into(), 32.into());
    entry.push_inst(mul.into());

    // normal i
    let dst = rg.gen_virtual_usual_reg();
    let lhs = rg.gen_virtual_usual_reg();
    let mul = MulInst::new(dst.into(), lhs.into(), 42.into());
    entry.push_inst(mul.into());

    // normal r
    let dst = rg.gen_virtual_usual_reg();
    let lhs = rg.gen_virtual_usual_reg();
    let rhs = rg.gen_virtual_usual_reg();
    let mul = MulInst::new(dst.into(), lhs.into(), rhs.into());
    entry.push_inst(mul.into());

    // left shift 5
    let dst = rg.gen_virtual_usual_reg();
    let lhs = rg.gen_virtual_usual_reg();
    let div = DivInst::new(dst.into(), lhs.into(), 16.into());
    entry.push_inst(div.into());

    // normal i
    let dst = rg.gen_virtual_usual_reg();
    let lhs = rg.gen_virtual_usual_reg();
    let div = DivInst::new(dst.into(), lhs.into(), 19.into());
    entry.push_inst(div.into());

    // normal i
    let dst = rg.gen_virtual_usual_reg();
    let lhs = rg.gen_virtual_usual_reg();
    let rhs = rg.gen_virtual_usual_reg();
    let div = DivInst::new(dst.into(), lhs.into(), rhs.into());
    entry.push_inst(div.into());

    entry.push_inst(Inst::Ret);

    let mut f = Func::new("mul_div".to_string(), vec![], entry);
    f.stack_allocator_mut().replace(ssa);
    f.reg_gener_mut().replace(rg);

    handle_illegal_inst(&mut f).unwrap();

    assert_debug_snapshot!(f, @r###"
    Func {
        name: "mul_div",
        args: [],
        ret: None,
        reg_gener: Some(
            RegGenerator {
                usual_counter: ParalCounter {
                    end: 100000000,
                    counter: 50,
                },
                float_counter: ParalCounter {
                    end: 100000000,
                    counter: 32,
                },
            },
        ),
        stack_allocator: Some(
            StackAllocator {
                alloc_from: 0,
            },
        ),
        entry: Block {
            label: "entry",
            insts: [
                Li(
                    LiInst(
                        Reg(
                            Reg {
                                id: 32,
                                is_usual: true,
                            },
                        ),
                        Imm(
                            Imm(
                                0,
                            ),
                        ),
                    ),
                ),
                Sll(
                    SllInst(
                        Reg(
                            Reg {
                                id: 34,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 35,
                                is_usual: true,
                            },
                        ),
                        Imm(
                            Imm(
                                5,
                            ),
                        ),
                        false,
                    ),
                ),
                Li(
                    LiInst(
                        Reg(
                            Reg {
                                id: 48,
                                is_usual: true,
                            },
                        ),
                        Imm(
                            Imm(
                                42,
                            ),
                        ),
                    ),
                ),
                Mul(
                    MulInst(
                        Reg(
                            Reg {
                                id: 36,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 37,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 48,
                                is_usual: true,
                            },
                        ),
                        false,
                    ),
                ),
                Mul(
                    MulInst(
                        Reg(
                            Reg {
                                id: 38,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 39,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 40,
                                is_usual: true,
                            },
                        ),
                        false,
                    ),
                ),
                SRA(
                    SraInst(
                        Reg(
                            Reg {
                                id: 41,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 42,
                                is_usual: true,
                            },
                        ),
                        Imm(
                            Imm(
                                4,
                            ),
                        ),
                    ),
                ),
                Li(
                    LiInst(
                        Reg(
                            Reg {
                                id: 49,
                                is_usual: true,
                            },
                        ),
                        Imm(
                            Imm(
                                19,
                            ),
                        ),
                    ),
                ),
                Div(
                    DivInst(
                        Reg(
                            Reg {
                                id: 43,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 44,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 49,
                                is_usual: true,
                            },
                        ),
                    ),
                ),
                Div(
                    DivInst(
                        Reg(
                            Reg {
                                id: 45,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 46,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 47,
                                is_usual: true,
                            },
                        ),
                    ),
                ),
                Ret,
            ],
            to_bbs: [],
        },
        other_bbs: [],
    }
    "###);
}
