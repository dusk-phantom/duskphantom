use std::process::Command;

use compiler::{backend::irs::*, fprintln};
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
        },
        other_bbs: [],
    }
    "###);
}

#[ignore = "This test need riscv64-linux-gnu-gcc to compile"]
#[test]
fn test_handle_long_jmp() {
    use compiler::backend::irs::*;
    use compiler::fprintln;
    let mut bb0 = Block::new("bb0".to_string());
    let mut bb1 = Block::new("bb1".to_string());
    let mut bb2 = Block::new("bb2".to_string());
    bb0.push_inst(JmpInst::new("bb2".into()).into());
    [(); 2000000]
        .iter()
        .for_each(|_| bb1.push_inst(LiInst::new(REG_A0.into(), 1.into()).into()));
    bb1.push_inst(Inst::Ret);
    bb2.push_inst(JmpInst::new("bb1".into()).into());
    let mut f = Func::new("main".to_string(), vec![], bb0);
    f.push_bb(bb1);
    f.push_bb(bb2);
    let mut mdl = Module::new("test");
    mdl.funcs.push(f);
    let asm_path = tempfile::Builder::new().suffix(".s").tempfile().unwrap();
    // let asm = mdl.gen_asm();
    // std::fs::write(asm_path.path(), asm).unwrap();
    // let cp = Command::new("riscv64-linux-gnu-gcc")
    //     .arg(asm_path.path())
    //     .arg("-o")
    //     .arg("/dev/null")
    //     .output()
    //     .unwrap();
    fprintln!("1.s", "{}", mdl.gen_asm());
}
