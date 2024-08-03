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

    phisicalize_func(&mut f).unwrap();

    assert_debug_snapshot!(f,@r###"
    Func {
        name: "test",
        args: [],
        ret: None,
        reg_gener: None,
        stack_allocator: Some(
            StackAllocator {
                alloc_from: 2432,
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
                    ),
                ),
                Li(
                    LiInst(
                        Reg(
                            Reg {
                                id: 9,
                                is_usual: true,
                            },
                        ),
                        Imm(
                            Imm(
                                2416,
                            ),
                        ),
                    ),
                ),
                Add(
                    AddInst(
                        Reg(
                            Reg {
                                id: 9,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 9,
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
                Sd(
                    SdInst(
                        Reg {
                            id: 9,
                            is_usual: true,
                        },
                        Imm(
                            0,
                        ),
                        Reg {
                            id: 9,
                            is_usual: true,
                        },
                    ),
                ),
                Li(
                    LiInst(
                        Reg(
                            Reg {
                                id: 9,
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
                                id: 9,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 9,
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
                            id: 9,
                            is_usual: true,
                        },
                    ),
                ),
                Li(
                    LiInst(
                        Reg(
                            Reg {
                                id: 9,
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
                                id: 9,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 9,
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
                            id: 9,
                            is_usual: true,
                        },
                    ),
                ),
                Li(
                    LiInst(
                        Reg(
                            Reg {
                                id: 9,
                                is_usual: true,
                            },
                        ),
                        Imm(
                            Imm(
                                2416,
                            ),
                        ),
                    ),
                ),
                Add(
                    AddInst(
                        Reg(
                            Reg {
                                id: 9,
                                is_usual: true,
                            },
                        ),
                        Reg(
                            Reg {
                                id: 9,
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
                Ld(
                    LdInst(
                        Reg {
                            id: 9,
                            is_usual: true,
                        },
                        Imm(
                            0,
                        ),
                        Reg {
                            id: 9,
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
