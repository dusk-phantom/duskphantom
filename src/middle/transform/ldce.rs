use std::{
    collections::{HashSet, VecDeque},
    pin::Pin,
};

use crate::middle::{
    analysis::{
        effect_analysis::EffectAnalysis,
        loop_tools::{LoopForest, LoopPtr},
    },
    ir::{instruction::InstType, BBPtr, InstPtr},
    IRBuilder,
};
use anyhow::{Ok, Result};

use super::loop_optimization::loop_forest_post_order;

type IRBuilderWraper = Pin<Box<IRBuilder>>;

pub struct LDCE<'a> {
    _ir_builder: &'a mut IRBuilderWraper,
    check_set: HashSet<InstPtr>,
    loop_bbs: HashSet<BBPtr>,
    effect_analysis: &'a EffectAnalysis,
}

impl<'a> LDCE<'a> {
    pub fn new(
        _ir_builder: &'a mut IRBuilderWraper,
        effect_analysis: &'a EffectAnalysis,
    ) -> LDCE<'a> {
        Self {
            _ir_builder,
            check_set: HashSet::new(),
            loop_bbs: HashSet::new(),
            effect_analysis,
        }
    }

    pub fn run(&mut self, forest: &mut LoopForest) -> Result<()> {
        loop_forest_post_order(forest, |x| self.ldce_one_loop(x))
    }

    fn ldce_one_loop(&mut self, lo: LoopPtr) -> Result<()> {
        self.loop_bbs.clear();
        {
            let mut queue = VecDeque::from([lo]);
            while let Some(lo) = queue.pop_back() {
                self.loop_bbs.extend(lo.blocks.iter());
                queue.extend(lo.sub_loops.iter());
            }
        }

        lo.blocks.iter().try_for_each(|&x| self.ldce_one_bb(x))
    }

    fn ldce_one_bb(&mut self, bb: BBPtr) -> Result<()> {
        let mut cur_set = HashSet::new();
        for inst in bb.iter() {
            if self.check_set.contains(&inst) {
                continue;
            }
            cur_set.extend(self.ldce_inst(inst)?);
        }
        cur_set.into_iter().for_each(|mut x| x.remove_self());
        Ok(())
    }

    fn ldce_inst(&mut self, inst: InstPtr) -> Result<HashSet<InstPtr>> {
        let check_user_out_loop = |inst: InstPtr| {
            inst.get_user()
                .iter()
                .any(|x| self.loop_bbs.contains(&x.get_parent_bb().unwrap()))
        };
        if check_user_out_loop(inst) {
            self.check_set.insert(inst);
            return Ok(HashSet::new());
        }

        let mut cur_set = HashSet::from([inst]);
        let mut queue = VecDeque::from([inst]);
        while let Some(inst) = queue.pop_back() {
            if !self.can_delete_inst(inst) || check_user_out_loop(inst) {
                self.check_set
                    .extend(cur_set.iter().chain(inst.get_user().iter()));
                return Ok(HashSet::new());
            }
            queue.extend(inst.get_user().iter().filter(|x| !cur_set.contains(x)));
            cur_set.extend(inst.get_user().iter());
        }

        self.check_set.extend(cur_set.iter());
        Ok(cur_set)
    }

    fn can_delete_inst(&self, inst: InstPtr) -> bool {
        let call_no_effect = !self.effect_analysis.has_effect(inst);
        let no_control_or_store = !matches!(
            inst.get_type(),
            InstType::Br | InstType::Ret | InstType::Store
        );
        call_no_effect && no_control_or_store
    }
}
