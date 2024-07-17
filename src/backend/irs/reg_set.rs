use anyhow::Ok;
use bitvec::{field::BitField, prelude};

use super::*;

pub struct RegSet {
    float: prelude::BitArray<[u64; 1000]>,
    usual: prelude::BitArray<[u64; 1000]>,
}
impl RegSet {
    #[allow(clippy::new_without_default)]
    #[inline]
    pub fn new() -> Self {
        Self {
            float: prelude::BitArray::new([0; 1000]),
            usual: prelude::BitArray::new([0; 1000]),
        }
    }
    /// NOTICE: This function will panic if reg id is out of range
    #[inline]
    pub fn quick_insert(&mut self, reg: &Reg) {
        let id = reg.id() as usize;
        if reg.is_usual() {
            self.usual.set(id, true);
        } else {
            self.float.set(id, true);
        }
    }
    #[inline]
    pub fn insert(&mut self, reg: &Reg) -> Result<()> {
        let id = reg.id() as usize;
        if reg.is_usual() {
            if self.usual.len() <= id {
                return Err(anyhow!("RegSet::insert: reg id out of range"));
            } else {
                self.usual.set(id, true);
            }
        } else if self.float.len() <= id {
            return Err(anyhow!("RegSet::insert: reg id out of range"));
        } else {
            self.float.set(id, true);
        }
        Ok(())
    }

    #[inline]
    pub fn merge(&mut self, other: &RegSet) {
        self.float |= &other.float;
        self.usual |= &other.usual;
    }

    #[inline]
    pub fn clear(&mut self) {
        self.float.store(0);
        self.usual.store(0);
    }
    #[inline]
    // NOTICE: This function will overflow
    // if the number of regs is greater than what usize can hold
    pub fn num_regs(&self) -> usize {
        self.float.count_ones() + self.usual.count_ones()
    }
    #[inline]
    // NOTICE: This function will overflow
    // if the number of regs is greater than what usize can hold
    pub fn num_regs_usual(&self) -> usize {
        self.usual.count_ones()
    }
    #[inline]
    // NOTICE: This function will overflow
    // if the number of regs is greater than what usize can hold
    pub fn num_regs_float(&self) -> usize {
        self.float.count_ones()
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.float.not_any() && self.usual.not_any()
    }
    #[inline]
    pub fn is_empty_usual(&self) -> bool {
        self.usual.not_any()
    }
    #[inline]
    pub fn is_empty_float(&self) -> bool {
        self.float.not_any()
    }

    #[inline]
    pub fn remove(&mut self, reg: &Reg) {
        let id = reg.id() as usize;
        if reg.is_usual() {
            self.usual.set(id, false);
        } else {
            self.float.set(id, false);
        }
    }
    #[inline]
    pub fn contains(&self, reg: &Reg) -> bool {
        let id = reg.id() as usize;
        if reg.is_usual() {
            self.usual[id]
        } else {
            self.float[id]
        }
    }
}

// impl iter::IntoIterator for RegSet

impl IntoIterator for RegSet {
    type Item = Reg;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        let r: Vec<Reg> = vec![];

        // TODO: Implement this

        r.into_iter()
    }
}

#[cfg(test)]
mod reg_set_tests {
    use super::*;
    #[test]
    fn test_reg_set() {
        let mut reg_set = RegSet::new();
        let reg = REG_A0;
        reg_set.quick_insert(&reg);
        assert!(reg_set.contains(&reg));
        reg_set.remove(&reg);
        assert!(!reg_set.contains(&reg));
    }
    // FIXME
    // #[test]
    // fn test_reg_set_iter() {
        
    // }

    #[test]
    fn test_reg_set_merge() {
        let mut reg_set1 = RegSet::new();
        let mut reg_set2 = RegSet::new();
        reg_set1.quick_insert(&REG_A0);
        reg_set1.quick_insert(&REG_FA0);
        reg_set2.quick_insert(&REG_A1);
        reg_set2.quick_insert(&REG_A2);
        reg_set2.quick_insert(&REG_A0);
        reg_set1.merge(&reg_set2);
        assert!(reg_set1.contains(&REG_A0));
        assert!(reg_set1.contains(&REG_A1));
        assert!(reg_set1.contains(&REG_A2));
        assert!(reg_set1.contains(&REG_FA0));
        assert!(reg_set1.num_regs() == 4);
        assert!(reg_set1.num_regs_usual() == 3);
        assert!(reg_set1.num_regs_float() == 1);
    }
    // FIXME
    // #[test]
    // fn test_reg_set_clear() {
    //     let mut reg_set = RegSet::new();
    //     reg_set.quick_insert(&REG_A0);
    //     reg_set.quick_insert(&REG_FA0);
    //     reg_set.clear();
    //     assert!(reg_set.is_empty());
    // }
    #[test]
    fn test_num_regs() {
        let mut reg_set = RegSet::new();
        reg_set.quick_insert(&REG_A0);
        reg_set.quick_insert(&REG_FA0);
        assert!(reg_set.num_regs() == 2);
        assert!(reg_set.num_regs_usual() == 1);
        assert!(reg_set.num_regs_float() == 1);
    }
    #[test]
    fn test_reg_set_empty() {
        let reg_set = RegSet::new();
        assert!(reg_set.is_empty());
        assert!(reg_set.is_empty_usual());
        assert!(reg_set.is_empty_float());
    }
    #[test]
    fn test_reg_set_insert() {
        let mut reg_set = RegSet::new();
        reg_set.insert(&REG_A0).unwrap();
        reg_set.insert(&REG_A1).unwrap();
        reg_set.insert(&REG_A2).unwrap();
        reg_set.insert(&REG_FA0).unwrap();
        assert!(reg_set.contains(&REG_A0));
        assert!(reg_set.contains(&REG_A1));
        assert!(reg_set.contains(&REG_A2));
        assert!(reg_set.contains(&REG_FA0));
    }
    // FIXME
    // #[test]
    // #[should_panic]
    // fn test_reg_set_insert_out_of_range() {
    //     let mut reg_set = RegSet::new();
    // }
}
