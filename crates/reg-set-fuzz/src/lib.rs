mod single;
pub use single::*;

use std::collections::{HashMap, HashSet};

use arbitrary::Arbitrary;
pub use compiler::backend::irs::*;
pub use reg_set::RegSet;

#[non_exhaustive]
#[derive(Arbitrary, Debug)]
pub enum Action {
    Insert(usize, MReg),
    Remove(usize, MReg),
    Clear(usize),
    Merge(usize, usize),
    Minus(usize, usize),
    Intersect(usize, usize),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MReg {
    pub id: u32,
    pub is_usual: bool,
}
impl From<Reg> for MReg {
    fn from(r: Reg) -> Self {
        MReg {
            id: r.id(),
            is_usual: r.is_usual(),
        }
    }
}
impl From<MReg> for Reg {
    fn from(r: MReg) -> Self {
        Reg::new(r.id, r.is_usual)
    }
}
impl From<&MReg> for Reg {
    fn from(r: &MReg) -> Self {
        (*r).into()
    }
}
impl From<&Reg> for MReg {
    fn from(r: &Reg) -> Self {
        (*r).into()
    }
}

// impl<'a> Arbitrary<'a> for Action{
//     fn arbitrary(u: &mut libfuzzer_sys::arbitrary::Unstructured<'a>) -> libfuzzer_sys::arbitrary::Result<Self> {
//         todo!()
//     }
// }

impl<'a> Arbitrary<'a> for MReg {
    fn arbitrary(
        u: &mut libfuzzer_sys::arbitrary::Unstructured<'a>,
    ) -> libfuzzer_sys::arbitrary::Result<Self> {
        let id: u16 = u.arbitrary()?;
        let id: u32 = id as u32 % 10_000;
        let is_usual: bool = u.arbitrary()?;
        let r = Reg::new(id, is_usual);
        Ok(r.into())
    }
}

#[derive(Default, Clone)]
pub struct Item {
    pub reg_set: RegSet,
    pub reg_hset: HashSet<Reg>,
}
pub fn apply_actions(rgs: &mut HashMap<usize, Item>, actions: Vec<Action>) -> HashMap<usize, Item> {
    let mut rgs = rgs.clone();
    for action in actions {
        apply_action(&mut rgs, action);
        assert!(check_valid(&rgs));
    }
    rgs
}

pub fn apply_action(rgs: &mut HashMap<usize, Item>, action: Action) {
    match action {
        Action::Insert(id, r) => {
            let rg = rgs.entry(id).or_default();
            rg.reg_set.insert(&r.into());
            rg.reg_hset.insert(r.into());
        }
        Action::Remove(id, r) => rgs.get_mut(&id).iter_mut().for_each(|i| {
            i.reg_set.remove(&r.into());
            i.reg_hset.remove(&r.into());
        }),
        Action::Clear(id) => {
            rgs.get_mut(&id).iter_mut().for_each(|i| {
                i.reg_set.clear();
                i.reg_hset.clear();
            });
        }
        Action::Merge(id1, id2) => {
            let rg2 = rgs.get(&id2).cloned().unwrap_or_default();
            rgs.get_mut(&id1).iter_mut().for_each(|i| {
                i.reg_set.merge(&rg2.reg_set);
                i.reg_hset.extend(rg2.reg_hset.iter().clone());
            });
        }
        Action::Minus(id1, id2) => {
            let rg2 = rgs.get(&id2).cloned().unwrap_or_default();
            rgs.get_mut(&id1).iter_mut().for_each(|i| {
                i.reg_set.minus(&rg2.reg_set);
                i.reg_hset.retain(|r| !rg2.reg_hset.contains(r));
            });
        }
        Action::Intersect(id1, id2) => {
            let rg2 = rgs.get(&id2).cloned().unwrap_or_default();
            let rg1 = rgs.entry(id1).or_default();
            rg1.reg_set.retain(|r| rg2.reg_set.contains(r));
            rg1.reg_hset.retain(|r| rg2.reg_hset.contains(r));
        }
    }
}

fn check_valid(rgs: &HashMap<usize, Item>) -> bool {
    for (_, rg) in rgs.iter() {
        if rg.reg_set.num_regs() != rg.reg_hset.len() {
            return false;
        }
        for r in rg.reg_hset.iter() {
            if !rg.reg_set.contains(r) {
                return false;
            }
        }
        for r in rg.reg_set.iter() {
            if !rg.reg_hset.contains(&r) {
                return false;
            }
        }
    }
    true
}
