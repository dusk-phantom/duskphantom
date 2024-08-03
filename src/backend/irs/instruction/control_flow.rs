use super::*;

#[derive(Debug, Clone)]
pub struct CallInst {
    dst: Label,
    uses: Vec<Reg>,
    def: Option<Reg>,
}
impl CallInst {
    pub fn new(dst: Label) -> Self {
        Self {
            dst,
            uses: vec![],
            def: None,
        }
    }
    pub fn func_name(&self) -> &Label {
        &self.dst
    }
    pub fn gen_asm(&self) -> String {
        let dst = self.func_name().gen_asm();
        format!("call {}", dst)
    }
    pub fn add_uses(&mut self, uses: &[Reg]) {
        self.uses.extend(uses.iter());
    }
    pub fn add_def(&mut self, def: Reg) {
        self.def = Some(def);
    }
}
impl RegUses for CallInst {
    fn uses(&self) -> Vec<&Reg> {
        self.uses.iter().collect()
    }
}
impl RegDefs for CallInst {
    fn defs(&self) -> Vec<&Reg> {
        self.def.iter().collect()
    }
}

impl_unary_inst!(JmpInst, "j");
impl_unary_inst!(TailInst, "tail");

impl_branch_inst!(BeqInst, "beq");
impl_branch_inst!(BneInst, "bne");
impl_branch_inst!(BltInst, "blt");
impl_branch_inst!(BleInst, "ble");
impl_branch_inst!(BgtInst, "bgt");
impl_branch_inst!(BgeInst, "bge");

pub trait ToBB {
    fn to_bb(&self) -> Result<&str>;
}

impl ToBB for JmpInst {
    fn to_bb(&self) -> Result<&str> {
        match &self.0 {
            Operand::Label(s) => Ok(s.as_str()),
            _ => Err(anyhow!("JmpInst: to_bb: not a label")),
        }
    }
}
impl ToBB for TailInst {
    fn to_bb(&self) -> Result<&str> {
        match &self.0 {
            Operand::Label(s) => Ok(s.as_str()),
            _ => Err(anyhow!("TailInst: to_bb: not a label")),
        }
    }
}

impl RegReplace for CallInst {}

mod convert_to_inst {
    use super::*;
    // inst for control flow
    impl_inst_convert!(JmpInst, Jmp);
    impl_inst_convert!(CallInst, Call);
    impl_inst_convert!(TailInst, Tail);
    impl_inst_convert!(BeqInst, Beq);
    impl_inst_convert!(BneInst, Bne);
    impl_inst_convert!(BltInst, Blt);
    impl_inst_convert!(BleInst, Ble);
    impl_inst_convert!(BgtInst, Bgt);
    impl_inst_convert!(BgeInst, Bge);
}
