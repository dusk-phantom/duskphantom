use reg_set::RegSet;

use super::*;

impl Func {
    pub fn def_then_use(func: &Func) -> HashMap<Reg, RegSet> {
        let mut use_then_def: HashMap<Reg, RegSet> = HashMap::new();
        for bb in func.iter_bbs() {
            let mut defs: Vec<Reg> = vec![];
            for inst in bb.insts() {
                for use_ in inst.uses() {
                    for def in defs.iter() {
                        use_then_def.entry(*use_).or_default().insert(def);
                    }
                }
                defs = inst.defs().into_iter().cloned().collect();
            }
        }
        use_then_def
    }
}
