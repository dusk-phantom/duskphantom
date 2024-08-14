use super::*;

#[derive(Debug, Arbitrary, Clone)]
pub enum SingleAction {
    Merge(Vec<MReg>),
    Remove(Vec<MReg>),
    Insert(Vec<MReg>),
    RetainInVec(Vec<MReg>),
    Clear,
}
fn prepare_for_merge(regs: &Vec<MReg>) -> RegSet {
    let mut rg = RegSet::new();
    for r in regs {
        rg.insert(r);
    }
    rg
}

pub fn apply_single_actions(rg: &mut RegSet, rge: &mut HashSet<Reg>, actions: Vec<SingleAction>) {
    for action in actions {
        apply_single_action(rg, rge, action);
        single_check(rg, rge);
    }
}
pub fn apply_single_action(rg: &mut RegSet, rge: &mut HashSet<Reg>, action: SingleAction) {
    match action {
        SingleAction::Insert(rs) => {
            for r in rs {
                rg.insert(&r);
                rge.insert(r.into());
            }
        }
        SingleAction::Remove(rs) => {
            for r in rs {
                rg.remove(&r);
                rge.remove(&r);
            }
        }
        SingleAction::Clear => {
            rg.clear();
            rge.clear();
        }
        SingleAction::Merge(rs) => {
            let rhs = prepare_for_merge(&rs);
            rg.merge(&rhs);
            for r in rs {
                rge.insert(r.into());
            }
        }
        SingleAction::RetainInVec(rs) => {
            rg.retain(|r| rs.contains(&MReg(*r)));
            rge.retain(|r| rs.contains(&MReg(*r)));
        }
    }
}

pub fn single_check(rg: &RegSet, rge: &HashSet<Reg>) {
    let mut rge2 = HashSet::new();
    for r in rg.iter() {
        rge2.insert(r);
    }
    assert_eq!(rge, &rge2);
}