use reg_set::RegSet;

use super::*;

impl Block {
    pub fn def_then_def(&self) -> HashMap<Reg, HashSet<Reg>> {
        let mut def_then_def: HashMap<Reg, HashSet<Reg>> = HashMap::new();
        let mut last_defs: Vec<Reg> = vec![];
        for inst in self.insts() {
            for cur_def in inst.defs() {
                for def in last_defs.iter().filter(|def| cur_def != *def) {
                    def_then_def.entry(*def).or_default().insert(*cur_def);
                }
            }
            last_defs = inst.defs().into_iter().cloned().collect();
        }
        def_then_def
    }

    /// 统计基本块中存在的所有def then def 关系的寄存器,返回一个有向边的集合
    /// 边的起点是某指令的def,终点是下一条指令的def的集合
    pub fn def_then_def2(&self) -> HashMap<Reg, RegSet> {
        let mut def_then_def: HashMap<Reg, RegSet> = HashMap::new();
        let mut last_defs: Vec<Reg> = vec![];
        for inst in self.insts() {
            for cur_def in inst.defs() {
                for def in last_defs.iter().filter(|def| cur_def != *def) {
                    def_then_def.entry(*def).or_default().insert(cur_def);
                }
            }
            last_defs = inst.defs().into_iter().cloned().collect();
        }
        def_then_def
    }
}

impl Func {
    pub fn def_then_def(&self) -> HashMap<Reg, HashSet<Reg>> {
        let mut def_then_def: HashMap<Reg, HashSet<Reg>> = HashMap::new();
        for bb in self.iter_bbs() {
            for (def, then_defs) in bb.def_then_def() {
                def_then_def.entry(def).or_default().extend(then_defs);
            }
        }
        def_then_def
    }

    /// 统计函数中存在的所有def then def 关系的寄存器,返回一个有向边的集合 而不是一个无向图
    pub fn def_then_def2(&self) -> HashMap<Reg, RegSet> {
        let mut def_then_def: HashMap<Reg, RegSet> = HashMap::new();
        for bb in self.iter_bbs() {
            for (def, then_defs) in bb.def_then_def2() {
                def_then_def.entry(def).or_default().merge(&then_defs);
            }
        }
        def_then_def
    }
}

#[cfg(test)]
mod tests {

    use crate::backend::irs::Reg;

    use super::{AddInst, Block, LiInst, MvInst};

    #[test]
    fn test_bb_def_then_def2() {
        let x32 = Reg::new(32, true);
        let x33 = Reg::new(33, true);
        let x34 = Reg::new(34, true);
        let mut bb = Block::new("".to_string());
        bb.push_inst(LiInst::new(x32.into(), 1.into()).into());
        bb.push_inst(MvInst::new(x33.into(), x32.into()).into());
        bb.push_inst(LiInst::new(x34.into(), 2.into()).into());
        bb.push_inst(AddInst::new(x32.into(), x33.into(), x34.into()).into());
        let def_then_def = bb.def_then_def2();
        assert!(def_then_def.len() == 3);
        assert!(def_then_def.get(&x32).unwrap().contains(&x33));
        assert!(def_then_def.get(&x33).unwrap().contains(&x34));
        assert!(def_then_def.get(&x34).unwrap().contains(&x32));
    }
}
