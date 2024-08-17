use super::*;
use insta::assert_debug_snapshot;

#[test]
fn test_int_arr1() {
    let var = Var::IntArr(ArrVar {
        name: "arr".to_string(),
        capacity: 2,
        init: vec![],
        is_const: false,
    });
    assert_debug_snapshot!(var.gen_asm(), @r###"".bss\n.align\t3\n.globl\tarr\n.type\tarr, @object\n.size\tarr, 8\narr:\n.zero\t8""###);
}

#[test]
fn test_int_arr2() {
    let var = Var::IntArr(ArrVar {
        name: "arr".to_string(),
        capacity: 99,
        init: vec![(0, 1), (3, 2), (4, 3)],
        is_const: false,
    });
    assert_debug_snapshot!(var.gen_asm(), @r###"".data\n.align\t3\n.globl\tarr\n.type\tarr, @object\n.size\tarr, 396\narr:\n.word\t0x1\n.zero\t8\n.word\t0x2\n.word\t0x3\n.zero\t376""###);
}
