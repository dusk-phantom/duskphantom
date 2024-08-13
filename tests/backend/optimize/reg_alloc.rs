use std::collections::{HashMap, HashSet};

use analysis::RegLives;
use compiler::{
    backend::{self, irs::*},
    fprintln, frontend, middle,
    utils::diff::diff,
};
use insta::{assert_debug_snapshot, assert_snapshot};
use reg_alloc::{free_fregs, free_iregs, reg_alloc};
pub fn backend_from_self(code: &str) -> Program {
    let f = frontend::parse(code).unwrap();
    let m = middle::r#gen(&f).unwrap();
    backend::from_self::gen_from_self(&m).unwrap()
}

pub fn find_func<'a>(b: &'a Program, name: &str) -> &'a Func {
    b.modules
        .first()
        .unwrap()
        .funcs
        .iter()
        .find(|f| f.name() == name)
        .unwrap()
}

#[test]
fn test_count_reg_inter_graph() {
    use compiler::backend::irs::*;
    // use insta::assert_debug_snapshot;
    fn construct_reg_alloc() -> Func {
        let mut entry = Block::new("entry".to_string());
        let ssa = StackAllocator::new();
        let mut rg = RegGenerator::new();

        // lla x33, sum
        let x32 = rg.gen_virtual_usual_reg();
        let addr = rg.gen_virtual_usual_reg();
        let lla = LlaInst::new(addr, "sum".into());
        entry.push_inst(lla.into());

        // lw x32, 0(x33)
        let lw = LwInst::new(x32, 0.into(), addr);
        entry.push_inst(lw.into());

        // lla x35, a
        let x34 = rg.gen_virtual_usual_reg();
        let addr = rg.gen_virtual_usual_reg();
        let lla = LlaInst::new(addr, "a".into());
        entry.push_inst(lla.into());

        // lw x34, 0(x35)
        let lw = LwInst::new(x34, 0.into(), addr);
        entry.push_inst(lw.into());

        // addw x36, x34, x32
        let x36 = rg.gen_virtual_usual_reg();
        let add = AddInst::new(x36.into(), x34.into(), x32.into());
        entry.push_inst(add.into());

        // lla x37, sum
        let addr = rg.gen_virtual_usual_reg();
        let lla = LlaInst::new(addr, "sum".into());
        entry.push_inst(lla.into());

        // sw x36, 0(x37)
        let sw = SwInst::new(x36, 0.into(), addr);
        entry.push_inst(sw.into());

        // call getA
        let mut call = CallInst::new("getA".into());
        call.add_def(REG_A0);
        entry.push_inst(call.into());

        // mv x38, a0
        let x38 = rg.gen_virtual_usual_reg();
        let mv = MvInst::new(x38.into(), REG_A0.into());
        entry.push_inst(mv.into());

        // lla x40, sum
        let x39 = rg.gen_virtual_usual_reg();
        let addr = rg.gen_virtual_usual_reg();
        let lla = LlaInst::new(addr, "sum".into());
        entry.push_inst(lla.into());

        // lw x39, 0(x40)
        let lw = LwInst::new(x39, 0.into(), addr);
        entry.push_inst(lw.into());

        // lla x42, a
        let x41 = rg.gen_virtual_usual_reg();
        let addr = rg.gen_virtual_usual_reg();
        let lla = LlaInst::new(addr, "a".into());
        entry.push_inst(lla.into());

        // lw x41, 0(x42)
        let lw = LwInst::new(x41, 0.into(), addr);
        entry.push_inst(lw.into());

        // addw x43, x41, x39
        let x43 = rg.gen_virtual_usual_reg();
        let add = AddInst::new(x43.into(), x41.into(), x39.into());
        entry.push_inst(add.into());

        // lla x44, sum
        let addr = rg.gen_virtual_usual_reg();
        let lla = LlaInst::new(addr, "sum".into());
        entry.push_inst(lla.into());

        // sw x43, 0(x44)
        let sw = SwInst::new(x43, 0.into(), addr);
        entry.push_inst(sw.into());

        entry.push_inst(Inst::Ret);

        let mut f = Func::new("f2".to_string(), vec![], entry);
        f.stack_allocator_mut().replace(ssa);
        f.reg_gener_mut().replace(rg);
        f
    }
    let f = construct_reg_alloc();
    let ig = Func::reg_interfere_graph(&f).unwrap();
    assert!(ig.contains_key(&Reg::new(38, true)));
    // println!("{}", f.gen_asm());
    // dbg!(&f);
}

#[test]
fn debug_rg_live() {
    // 从相对路径获取字符串
    let code = include_str!("94_loop3.c");
    let b = backend_from_self(code);
    let f = find_func(&b, "loop3");
    let lu1 = Func::reg_live_use(f);
    let lu2 = Func::reg_live_use2(f);
    let lu2 = lu2
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect::<HashSet<_>>()))
        .collect::<HashMap<_, _>>();
    assert_eq!(lu1, lu2);

    let ld1 = Func::reg_live_def(f);
    let ld2 = Func::reg_live_def2(f);
    let ld2 = ld2
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect::<HashSet<_>>()))
        .collect::<HashMap<_, _>>();
    assert_eq!(ld1, ld2);

    let rl1 = Func::reg_lives(f).unwrap();
    let rl2 = Func::reg_lives2(f).unwrap();

    let rl2: RegLives = rl2.into();
    assert_eq!(format!("{:?}", rl1), format!("{:?}", rl2));
}
