use super::{gen_asm::GenTool, *};
use crate::config::CONFIG;

#[allow(unused)]
pub struct Block {
    label: String,
    insts: Vec<inst::Inst>,
    // Vec<(inst index, label)>, to be filled after all insts are generated
    to_bbs: Vec<(usize, String)>,
}

impl Block {
    pub fn label(&self) -> &str {
        self.label.as_str()
    }
    pub fn new(label: String) -> Block {
        Block {
            label,
            insts: Vec::new(),
            to_bbs: Vec::new(),
        }
    }
    pub fn push_inst(&mut self, inst: inst::Inst) {
        self.insts.push(inst);
    }
    pub fn gen_asm(&self) -> String {
        let label = self.label.as_str();
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(CONFIG.num_parallel_for_inst_gen_asm)
            .build()
            .unwrap();
        let insts = thread_pool.install(|| {
            self.insts
                .par_iter()
                .map(|inst| inst.gen_asm())
                .collect::<Vec<String>>()
                .join("\n")
        });
        GenTool::gen_bb(label, insts.as_str())
    }
}