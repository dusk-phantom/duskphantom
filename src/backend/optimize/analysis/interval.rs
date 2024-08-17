use super::*;
use rustc_hash::FxHashSet;
use std::ops::Range;

pub struct RegIntervalCounter {
    pub live_after: HashMap<String, Vec<FxHashSet<Reg>>>,
    pub intervals: HashMap<String, HashMap<Range<usize>, FxHashSet<Reg>>>,
}

impl RegIntervalCounter {
    #[allow(unused)]
    /// interval analysis
    pub fn count(func: &Func) -> Result<Self> {
        let reg_lives = Func::reg_lives(func)?;
        for bb in func.iter_bbs() {
            let mut live_afters = vec![];
            let live_in = reg_lives.live_ins(bb);
            let live_out = reg_lives.live_outs(bb);
            let mut live_after: FxHashSet<Reg> = live_out.clone();
            live_afters.push(live_after.clone());
            for inst in bb.insts().iter().rev() {
                for reg in inst.defs() {
                    live_after.remove(reg);
                }
                for reg in inst.uses() {
                    live_after.insert(*reg);
                }
                live_afters.push(live_after.clone());
            }
        }
        unimplemented!();
    }

    #[allow(unused)]
    /// FIXME: test needed
    /// get registers which born between from and to,including from and to
    pub fn occur_between(&self, bb: &str, from: usize, mut to: usize) -> Result<FxHashSet<Reg>> {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[allow(unused)]
    fn construct_f() -> Func {
        let is_usual = true;
        let x33 = Reg::new(33, is_usual);
        let x34 = Reg::new(34, is_usual);
        let x35 = Reg::new(35, is_usual);
        let x36 = Reg::new(36, is_usual);
        // entry:
        // li x33,33
        // li x34,11
        // addi x35,x33,x34
        // beq x35,zero,bb_1
        // j bb2
        let mut entry = Block::new("entry".to_string());
        entry.push_inst(LiInst::new(x33.into(), 33.into()).into());
        entry.push_inst(LiInst::new(x34.into(), 11.into()).into());
        entry.push_inst(AddInst::new(x35.into(), x33.into(), x34.into()).into());
        entry.push_inst(BeqInst::new(x35, REG_ZERO, "bb1".into()).into());
        entry.push_inst(JmpInst::new("bb2".into()).into());

        // bb1:
        // mv x35,x34
        // mv a0,x35
        // ret
        let mut bb1 = Block::new("bb1".to_string());
        bb1.push_inst(MvInst::new(x35.into(), x34.into()).into());
        bb1.push_inst(MvInst::new(REG_A0.into(), x35.into()).into());
        bb1.push_inst(Inst::Ret);

        // bb2:
        // addi x35,x35,2
        // mv x36,x35
        // mv xa0,x36
        // ret
        let mut bb2 = Block::new("bb2".to_string());
        bb2.push_inst(AddInst::new(x35.into(), x35.into(), 2.into()).into());
        bb2.push_inst(MvInst::new(x36.into(), x35.into()).into());
        bb2.push_inst(MvInst::new(REG_A0.into(), x36.into()).into());
        bb2.push_inst(Inst::Ret);

        let mut func = Func::new("test".to_string(), vec![], entry);
        func.push_bb(bb1);
        func.push_bb(bb2);
        func
    }

    // #[test]
    // fn test1() {
    //     let func = construct_f();
    //     let counter = RegIntervalCounter::count(&func).unwrap();

    //     assert_debug_snapshot!(&counter,@r###"
    //     RegIntervalCounter {
    //         bb1: [
    //             RegSet {
    //                 usual: [
    //                     Reg {
    //                         id: 34,
    //                         is_usual: true,
    //                     },
    //                 ],
    //                 float: [],
    //             },
    //             RegSet {
    //                 usual: [
    //                     Reg {
    //                         id: 34,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 35,
    //                         is_usual: true,
    //                     },
    //                 ],
    //                 float: [],
    //             },
    //             RegSet {
    //                 usual: [
    //                     Reg {
    //                         id: 10,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 34,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 35,
    //                         is_usual: true,
    //                     },
    //                 ],
    //                 float: [],
    //             },
    //             RegSet {
    //                 usual: [
    //                     Reg {
    //                         id: 10,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 34,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 35,
    //                         is_usual: true,
    //                     },
    //                 ],
    //                 float: [],
    //             },
    //         ],
    //         bb2: [
    //             RegSet {
    //                 usual: [
    //                     Reg {
    //                         id: 34,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 35,
    //                         is_usual: true,
    //                     },
    //                 ],
    //                 float: [],
    //             },
    //             RegSet {
    //                 usual: [
    //                     Reg {
    //                         id: 34,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 35,
    //                         is_usual: true,
    //                     },
    //                 ],
    //                 float: [],
    //             },
    //             RegSet {
    //                 usual: [
    //                     Reg {
    //                         id: 34,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 35,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 36,
    //                         is_usual: true,
    //                     },
    //                 ],
    //                 float: [],
    //             },
    //             RegSet {
    //                 usual: [
    //                     Reg {
    //                         id: 10,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 34,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 35,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 36,
    //                         is_usual: true,
    //                     },
    //                 ],
    //                 float: [],
    //             },
    //             RegSet {
    //                 usual: [
    //                     Reg {
    //                         id: 10,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 34,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 35,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 36,
    //                         is_usual: true,
    //                     },
    //                 ],
    //                 float: [],
    //             },
    //         ],
    //         entry: [
    //             RegSet {
    //                 usual: [
    //                     Reg {
    //                         id: 0,
    //                         is_usual: true,
    //                     },
    //                 ],
    //                 float: [],
    //             },
    //             RegSet {
    //                 usual: [
    //                     Reg {
    //                         id: 0,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 33,
    //                         is_usual: true,
    //                     },
    //                 ],
    //                 float: [],
    //             },
    //             RegSet {
    //                 usual: [
    //                     Reg {
    //                         id: 0,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 33,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 34,
    //                         is_usual: true,
    //                     },
    //                 ],
    //                 float: [],
    //             },
    //             RegSet {
    //                 usual: [
    //                     Reg {
    //                         id: 0,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 33,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 34,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 35,
    //                         is_usual: true,
    //                     },
    //                 ],
    //                 float: [],
    //             },
    //             RegSet {
    //                 usual: [
    //                     Reg {
    //                         id: 0,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 33,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 34,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 35,
    //                         is_usual: true,
    //                     },
    //                 ],
    //                 float: [],
    //             },
    //             RegSet {
    //                 usual: [
    //                     Reg {
    //                         id: 0,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 33,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 34,
    //                         is_usual: true,
    //                     },
    //                     Reg {
    //                         id: 35,
    //                         is_usual: true,
    //                     },
    //                 ],
    //                 float: [],
    //             },
    //         ],
    //     }
    //     "###);
    // }

    #[test]
    /// FIXME
    fn test() {
        // let func = construct_f();
        // let counter = RegIntervalCounter::count(&func).unwrap();
        // let x33 = Reg::new(33, true);
        // let x34 = Reg::new(34, true);
        // let x35 = Reg::new(35, true);
        // let x36 = Reg::new(36, true);
        // let entry = func.find_bb("entry").unwrap();
        // assert_eq!(
        //     counter
        //         .occur_between(entry.label(), 0, entry.insts().len())
        //         .unwrap(),
        //     vec![x33, x34, x35].into_iter().collect()
        // );

        // let bb1 = func.find_bb("bb1").unwrap();
        // assert_eq!(
        //     counter
        //         .occur_between(bb1.label(), 0, bb1.insts().len())
        //         .unwrap(),
        //     vec![x35, REG_A0].into_iter().collect()
        // );

        // let bb2 = func.find_bb("bb2").unwrap();
        // assert_eq!(
        //     counter
        //         .occur_between(bb2.label(), 0, bb2.insts().len())
        //         .unwrap(),
        //     vec![x35, x36, REG_A0].into_iter().collect()
        // );
    }
}
