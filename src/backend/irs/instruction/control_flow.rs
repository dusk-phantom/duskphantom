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

// #[macro_export]
// macro_rules! impl_unary_inst {
//     ($ty_name:ident,$inst_name:expr) => {
//         #[derive(Clone, Debug)]
//         pub struct $ty_name(Operand);
//         impl $ty_name {
//             pub fn new(dst: Operand) -> Self {
//                 Self(dst)
//             }
//             pub fn dst(&self) -> &Operand {
//                 &self.0
//             }
//             pub fn dst_mut(&mut self) -> &mut Operand {
//                 &mut self.0
//             }
//             pub fn gen_asm(&self) -> String {
//                 let dst = self.dst().gen_asm();
//                 format!("{} {}", $inst_name, dst)
//             }
//         }

//         impl RegReplace for $ty_name {
//             fn replace_def(&mut self, from: Reg, to: Reg) -> Result<()> {
//                 if let Operand::Reg(reg) = self.dst_mut() {
//                     if *reg == from {
//                         *reg = to;
//                     }
//                 }
//                 Ok(())
//             }
//             fn replace_use(&mut self, from: Reg, to: Reg) -> Result<()> {
//                 if let Operand::Reg(reg) = self.dst_mut() {
//                     if *reg == from {
//                         *reg = to;
//                     }
//                 }
//                 Ok(())
//             }
//         }
//     };
// }
// impl_unary_inst!(JmpInst, "j");
#[derive(Clone, Debug)]
pub enum JmpInst {
    Short(Operand),
    Long(Operand, Reg),
}
impl JmpInst {
    pub fn new(dst: Operand) -> Self {
        Self::Short(dst)
    }
    pub fn dst(&self) -> &Operand {
        match self {
            Self::Short(op) => op,
            Self::Long(op, _) => op,
        }
    }
    pub fn dst_mut(&mut self) -> &mut Operand {
        match self {
            Self::Short(op) => op,
            Self::Long(op, _) => op,
        }
    }
    pub fn gen_asm(&self) -> String {
        match self {
            Self::Short(dst) => format!("j {}", dst.gen_asm()),
            Self::Long(dst, mid_reg) => {
                let dst = dst.gen_asm();
                let mid_reg = mid_reg.gen_asm();
                let mut asm = String::new();
                asm.push_str(&format!("lla {},{}\n", mid_reg, dst));
                asm.push_str(&format!("jalr zero,{},0", mid_reg));
                asm
            }
        }
    }
    pub fn set_long(&mut self, mid_reg: Reg) {
        *self = match self {
            Self::Short(dst) => Self::Long(dst.clone(), mid_reg),
            Self::Long(dst, _) => Self::Long(dst.clone(), mid_reg),
        }
    }
}
impl RegDefs for JmpInst {
    fn defs(&self) -> Vec<&Reg> {
        match self {
            Self::Short(_) => vec![],
            Self::Long(_, reg) => vec![reg],
        }
    }
}
impl RegUses for JmpInst {
    fn uses(&self) -> Vec<&Reg> {
        vec![&REG_ZERO]
    }
}
impl RegReplace for JmpInst {
    fn replace_def(&mut self, from: Reg, to: Reg) -> Result<()> {
        if let Self::Long(_, reg) = self {
            if *reg == from {
                *reg = to;
            }
        }
        Ok(())
    }
}

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
        match &self.dst() {
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
