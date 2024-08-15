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
        reg_set_fuzz::SingleAction::Merge(vec![Reg::new(0, false).into()]),
    );
    single_check(&rg, &rge);
}

#[test]
fn tt2() {
    let mut rg = RegSet::new();
    let mut rge: HashSet<Reg> = HashSet::new();
    let actions = [
        SingleAction::Insert(vec![Reg::new(372, false).into()]),
        SingleAction::RetainInVec(vec![Reg::new(0, false).into()]),
    ];
    apply_single_action(&mut rg, &mut rge, actions[0].clone());
    apply_single_action(&mut rg, &mut rge, actions[1].clone());
    // dbg!(&rg);
    // dbg!(&rge);
    single_check(&rg, &rge);
}

#[test]
fn tt3() {
    use SingleAction::*;
    let action = Merge(vec![
        MReg {
            id: 7,
            is_usual: false,
        },
        MReg {
            id: 0,
            is_usual: false,
        },
    ]);
    let mut rs = RegSet::new();
    let mut rse = HashSet::new();
    println!("{}", std::mem::size_of_val(&rs));
    print!("{}", std::mem::size_of_val(&rse));
    dbg!(&rs.float, &rs.usual);
    dbg!(&rs, &rse);
    apply_single_action(&mut rs, &mut rse, action);
    single_check(&rs, &rse);
}
