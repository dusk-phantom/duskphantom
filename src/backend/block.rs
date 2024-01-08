use super::{asm::GenTool, *};
pub struct Block {
    label: String,
    insts: Vec<inst::Inst>,
}

impl Block {
    pub fn label(&self) -> &str {
        self.label.as_str()
    }
    pub fn gen_asm(&self) -> String {
        let label = self.label.as_str();
        let insts = self
            .insts
            .par_iter()
            .map(|inst| inst.gen_asm())
            .collect::<Vec<String>>()
            .join("\n");
        GenTool::gen_bb(label, insts.as_str())
    }
}
