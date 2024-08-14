use reg_set::RegSet;

use super::*;

impl Block {
    pub fn def_then_use(&self) -> HashMap<Reg, RegSet> {
        let mut use_then_def: HashMap<Reg, RegSet> = HashMap::new();
        let mut defs: Vec<Reg> = vec![];
        for inst in self.insts() {
            for use_ in inst.uses() {
                for def in defs.iter() {
                    use_then_def.entry(*use_).or_default().insert(def);
                }
            }
            defs = inst.defs().into_iter().cloned().collect();
        }
        use_then_def
    }
}

impl Func {
    pub fn def_then_use(&self) -> HashMap<Reg, RegSet> {
        let mut use_then_def: HashMap<Reg, RegSet> = HashMap::new();
        for bb in self.iter_bbs() {
            use_then_def.extend(bb.def_then_use());
        }
        use_then_def
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use crate::backend::irs::Reg;

    use super::{AddInst, Block, LiInst, MvInst};

    #[test]
    fn test_bb_def_then_use() {
        let x32 = Reg::new(32, true);
        let x33 = Reg::new(33, true);
        let x34 = Reg::new(34, true);
        let x35 = Reg::new(35, true);
        let mut bb = Block::new("".to_string());
        bb.push_inst(LiInst::new(x32.into(), 1.into()).into());
        bb.push_inst(MvInst::new(x33.into(), x32.into()).into());
        bb.push_inst(LiInst::new(x34.into(), 2.into()).into());
        bb.push_inst(AddInst::new(x35.into(), x33.into(), x34.into()).into());
        assert_debug_snapshot!(bb.def_then_use(),@r###"
        {
            Reg {
                id: 33,
                is_usual: true,
            }: RegSet {
                usual: [
                    Reg {
                        id: 34,
                        is_usual: true,
                    },
                ],
                float: [],
            },
            Reg {
                id: 32,
                is_usual: true,
            }: RegSet {
                usual: [
                    Reg {
                        id: 32,
                        is_usual: true,
                    },
                ],
                float: [],
            },
            Reg {
                id: 34,
                is_usual: true,
            }: RegSet {
                usual: [
                    Reg {
                        id: 34,
                        is_usual: true,
                    },
                ],
                float: [],
            },
        }
        "###);
    }
}
