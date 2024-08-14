use std::collections::HashSet;

use reg_set_fuzz::*;

fn main() {
    let mut rg = RegSet::new();
    let mut rge = HashSet::new();
    let actions = Vec::new();
    apply_single_actions(&mut rg, &mut rge, actions);

    println!("Hello, world!");
}

#[test]
fn tt() {
    let mut rg = RegSet::new();
    let mut rge: HashSet<Reg> = HashSet::new();
    apply_single_action(
        &mut rg,
        &mut rge,
        reg_set_fuzz::SingleAction::Merge(vec![reg_set_fuzz::MReg(Reg::new(0, false))]),
    );
    single_check(&rg, &rge);
}

#[test]
fn tt2() {
    let mut rg = RegSet::new();
    let mut rge: HashSet<Reg> = HashSet::new();
    let actions = [
        SingleAction::Insert(vec![MReg(Reg::new(372, false))]),
        SingleAction::RetainInVec(vec![MReg(Reg::new(0, false))]),
    ];
    apply_single_action(&mut rg, &mut rge, actions[0].clone());
    apply_single_action(&mut rg, &mut rge, actions[1].clone());
    // dbg!(&rg);
    // dbg!(&rge);
    single_check(&rg, &rge);
}
