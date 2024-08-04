use super::{gen_asm::GenTool, *};

#[allow(unused)]
#[derive(Debug)]
pub struct Block {
    label: String,
    insts: Vec<Inst>,
    // Vec<(inst index, label)>, to be filled after all insts are generated
    to_bbs: Vec<(usize, String)>,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            label: "default".to_string(),
            insts: vec![],
            to_bbs: vec![],
        }
    }
}

impl Block {
    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    pub fn set_label(&mut self, label: &str) {
        self.label = label.to_string();
    }

    pub fn new(label: String) -> Block {
        Block {
            label,
            insts: Vec::new(),
            to_bbs: Vec::new(),
        }
    }

    pub fn push_inst(&mut self, inst: Inst) {
        self.insts.push(inst);
    }

    pub fn extend_insts(&mut self, insts: Vec<Inst>) {
        self.insts.extend(insts);
    }

    pub fn gen_asm(&self) -> String {
        let label = self.label.as_str();

        let insts = self
            .insts
            .iter()
            .map(|inst| inst.gen_asm())
            .collect::<Vec<String>>()
            .join("\n");
        GenTool::gen_bb(label, insts.as_str())
    }

    pub fn insts(&self) -> &Vec<Inst> {
        &self.insts
    }

    pub fn insts_mut(&mut self) -> &mut Vec<Inst> {
        &mut self.insts
    }

    pub fn insert_before_term(&mut self, inst: Inst) -> Result<()> {
        let is_term = |inst: &Inst| {
            matches!(
                inst,
                Inst::Jmp(_)
                    | Inst::Tail(_)
                    | Inst::Ret
                    | Inst::Beq(_)
                    | Inst::Bne(_)
                    | Inst::Blt(_)
                    | Inst::Ble(_)
                    | Inst::Bgt(_)
                    | Inst::Bge(_)
            )
        };
        let first_term = self.insts.iter().position(is_term);
        if let Some(first_term) = first_term {
            self.insts.insert(first_term, inst);
        } else {
            unreachable!("no term inst in block");
        }
        Ok(())
    }
}
