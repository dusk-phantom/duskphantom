use bitvec::prelude;

use super::*;

#[derive(Debug, Clone)]
pub struct RegSet {
    float: prelude::BitVec,
    usual: prelude::BitVec,
}

impl Eq for RegSet {}
impl PartialEq for RegSet {
    fn eq(&self, other: &Self) -> bool {
        self.float == other.float && self.usual == other.usual
    }
}

impl Default for RegSet {
    fn default() -> Self {
        Self::new()
    }
}
impl RegSet {
    #[inline]
    pub fn new() -> Self {
        Self {
            float: prelude::BitVec::new(),
            usual: prelude::BitVec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            float: prelude::BitVec::with_capacity(capacity),
            usual: prelude::BitVec::with_capacity(capacity),
        }
    }

    pub fn retain(&mut self, f: impl Fn(&Reg) -> bool) {
        let mut float = prelude::BitVec::new();
        let mut usual = prelude::BitVec::new();
        for r in self.iter() {
            if f(&r) {
                if r.is_usual() {
                    usual.push(true);
                } else {
                    float.push(true);
                }
            }
        }
        self.float = float;
        self.usual = usual;
    }

    pub fn insert(&mut self, reg: &Reg) {
        let id = reg.id() as usize;
        if reg.is_usual() {
            if self.usual.len() <= id {
                self.usual.resize(id + 100, false);
            }
            self.usual.set(id, true);
        } else {
            if self.float.len() <= id {
                self.float.resize(id + 100, false);
            }
            self.float.set(id, true);
        }
    }

    pub fn merge(&mut self, other: &RegSet) {
        self.float |= &other.float;
        self.usual |= &other.usual;
    }
    pub fn clear(&mut self) {
        self.float.clear();
        self.usual.clear();
    }

    // NOTICE: This function will overflow
    // if the number of regs is greater than what usize can hold
    pub fn num_regs(&self) -> usize {
        self.float.count_ones() + self.usual.count_ones()
    }

    // NOTICE: This function will overflow
    // if the number of regs is greater than what usize can hold
    pub fn num_regs_usual(&self) -> usize {
        self.usual.count_ones()
    }

    // NOTICE: This function will overflow
    // if the number of regs is greater than what usize can hold
    pub fn num_regs_float(&self) -> usize {
        self.float.count_ones()
    }

    pub fn is_empty(&self) -> bool {
        self.float.not_any() && self.usual.not_any()
    }

    pub fn has_none_usual(&self) -> bool {
        self.usual.not_any()
    }

    pub fn has_none_float(&self) -> bool {
        self.float.not_any()
    }

    pub fn remove(&mut self, reg: &Reg) {
        let id = reg.id() as usize;
        if reg.is_usual() {
            if self.usual.len() <= id {
                return;
            }
            self.usual.set(id, false);
        } else {
            if self.float.len() <= id {
                return;
            }
            self.float.set(id, false);
        }
    }

    pub fn contains(&self, reg: &Reg) -> bool {
        let id = reg.id() as usize;
        if reg.is_usual() {
            if self.usual.len() <= id {
                return false;
            }
            self.usual[id]
        } else {
            if self.float.len() <= id {
                return false;
            }
            self.float[id]
        }
    }

    /// An iterator visiting all float regs in id ascending order.
    pub fn iter_float(&self) -> impl IntoIterator<Item = Reg> + '_ {
        self.float
            .iter_ones()
            .into_iter()
            .map(|idx| Reg::new(idx as u32, false))
    }

    /// An iterator visiting all usual regs in id ascending order.
    pub fn iter_usual(&self) -> impl IntoIterator<Item = Reg> + '_ {
        self.usual
            .iter_ones()
            .into_iter()
            .map(|idx| Reg::new(idx as u32, true))
    }

    /// An iterator visiting all regs, first usual then float ,and in id ascending order each part.
    /// Note that float reg and usual reg may have the same id.
    pub fn iter(&self) -> impl IntoIterator<Item = Reg> + '_ {
        let it_u = self.iter_usual();
        let it_f = self.iter_float();
        it_u.into_iter().chain(it_f)
    }
}

impl IntoIterator for RegSet {
    type Item = Reg;
    type IntoIter = std::vec::IntoIter<Reg>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter().into_iter().collect::<Vec<Reg>>().into_iter()
    }
}

impl std::iter::FromIterator<Reg> for RegSet {
    fn from_iter<I: IntoIterator<Item = Reg>>(iter: I) -> Self {
        let mut reg_set = RegSet::new();
        for reg in iter {
            reg_set.insert(&reg);
        }
        reg_set
    }
}

#[cfg(test)]
mod reg_set_tests {

    use std::collections::HashSet;

    use super::*;
    #[test]
    fn test_reg_set() {
        let mut reg_set = RegSet::new();
        let reg = REG_A0;
        reg_set.insert(&reg);
        assert!(reg_set.contains(&reg));
        reg_set.remove(&reg);
        assert!(!reg_set.contains(&reg));
    }

    #[test]
    /// reg_set iter
    fn test_reg_set_iter() {
        let mut reg_set = RegSet::new();
        reg_set.insert(&REG_A0);
        reg_set.insert(&REG_A2);
        reg_set.insert(&REG_A1);
        reg_set.insert(&REG_FA1);
        reg_set.insert(&REG_FA0);
        let iter = reg_set.iter();
        let regs: HashSet<Reg> = iter.into_iter().collect();
        assert_eq!(
            regs,
            vec![REG_A0, REG_A1, REG_A2, REG_FA0, REG_FA1]
                .into_iter()
                .collect()
        );

        let regs: HashSet<Reg> = reg_set.iter_float().into_iter().collect();
        assert_eq!(regs, vec![REG_FA0, REG_FA1].into_iter().collect());

        let regs: HashSet<Reg> = reg_set.iter_usual().into_iter().collect();
        assert_eq!(regs, vec![REG_A0, REG_A1, REG_A2].into_iter().collect());
    }

    #[test]
    fn test_into_iter() {
        let mut reg_set = RegSet::new();
        reg_set.insert(&REG_A0);
        reg_set.insert(&REG_A2);
        reg_set.insert(&REG_A1);
        reg_set.insert(&REG_FA1);
        reg_set.insert(&REG_FA0);
        let regs: HashSet<Reg> = reg_set.into_iter().collect();
        assert_eq!(
            regs,
            vec![REG_A0, REG_A1, REG_A2, REG_FA0, REG_FA1]
                .into_iter()
                .collect()
        );
    }

    #[test]
    fn test_reg_set_merge() {
        let mut reg_set1 = RegSet::new();
        let mut reg_set2 = RegSet::new();
        reg_set1.insert(&REG_A0);
        reg_set1.insert(&REG_FA0);

        reg_set2.insert(&REG_A1);
        reg_set2.insert(&REG_A2);
        reg_set2.insert(&REG_A0);

        reg_set1.merge(&reg_set2);
        assert!(reg_set1.contains(&REG_A0));
        assert!(reg_set1.contains(&REG_A1));
        assert!(reg_set1.contains(&REG_A2));
        assert!(reg_set1.contains(&REG_FA0));
        assert!(reg_set1.num_regs() == 4);
        assert!(reg_set1.num_regs_usual() == 3);
        assert!(reg_set1.num_regs_float() == 1);
    }

    #[test]
    fn test_reg_set_clear() {
        let mut reg_set = RegSet::new();
        reg_set.insert(&REG_A0);
        reg_set.insert(&REG_FA0);
        reg_set.clear();
        assert!(reg_set.is_empty());
    }

    #[test]
    fn test_num_regs() {
        let mut reg_set = RegSet::new();
        reg_set.insert(&REG_A0);
        reg_set.insert(&REG_FA0);
        assert!(reg_set.num_regs() == 2);
        assert!(reg_set.num_regs_usual() == 1);
        assert!(reg_set.num_regs_float() == 1);
    }

    #[test]
    fn test_reg_set_empty() {
        let reg_set = RegSet::new();
        assert!(reg_set.is_empty());
        assert!(reg_set.has_none_usual());
        assert!(reg_set.has_none_float());
    }

    #[test]
    fn test_reg_set_insert() {
        let mut reg_set = RegSet::new();
        reg_set.insert(&REG_A0);
        reg_set.insert(&REG_A1);
        reg_set.insert(&REG_A2);
        reg_set.insert(&REG_FA0);
        assert!(reg_set.contains(&REG_A0));
        assert!(reg_set.contains(&REG_A1));
        assert!(reg_set.contains(&REG_A2));
        assert!(reg_set.contains(&REG_FA0));
    }

    #[test]
    fn test_reg_set_eq() {
        let mut reg_set = RegSet::with_capacity(1000);
        let mut reg_set2 = RegSet::with_capacity(1);
        reg_set.insert(&REG_A0);
        reg_set.insert(&REG_A1);
        reg_set.insert(&REG_A2);

        reg_set2.insert(&REG_A0);
        reg_set2.insert(&REG_A1);
        reg_set2.insert(&REG_A2);

        assert_eq!(reg_set, reg_set2);

        reg_set.remove(&REG_A0);
        assert_ne!(reg_set, reg_set2);

        reg_set2.remove(&REG_A0);
        assert_eq!(reg_set, reg_set2);
    }
}
