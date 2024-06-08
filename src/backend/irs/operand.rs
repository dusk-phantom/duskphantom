use crate::utils::paral_counter::ParalCounter;
use once_cell::sync::Lazy;
use std::ops::Deref;

#[derive(Clone)]
pub enum Operand {
    Reg(Reg),
    Imm(Imm),
    Fmm(Fmm),
    Label(Label),
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Reg {
    id: u32,
    is_usual: bool,
}
impl Reg {
    pub const fn new(id: u32, is_usual: bool) -> Self {
        Self { id, is_usual }
    }
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn is_usual(&self) -> bool {
        self.is_usual
    }
}
#[derive(Clone)]
pub struct Imm(i64);
#[derive(Clone)]
pub struct Fmm(f64);
#[derive(Clone)]
pub struct Label(String);

// impl from for Operand
impl From<i64> for Imm {
    fn from(val: i64) -> Self {
        Self(val)
    }
}
impl From<f64> for Fmm {
    fn from(val: f64) -> Self {
        Self(val)
    }
}
impl From<String> for Label {
    fn from(val: String) -> Self {
        Self(val)
    }
}
impl From<&str> for Label {
    fn from(val: &str) -> Self {
        Self(val.to_string())
    }
}
impl From<Reg> for Operand {
    fn from(val: Reg) -> Self {
        Self::Reg(val)
    }
}
impl From<Imm> for Operand {
    fn from(val: Imm) -> Self {
        Self::Imm(val)
    }
}
impl From<Fmm> for Operand {
    fn from(val: Fmm) -> Self {
        Self::Fmm(val)
    }
}
impl From<Label> for Operand {
    fn from(val: Label) -> Self {
        Self::Label(val)
    }
}
impl From<&str> for Operand {
    fn from(val: &str) -> Self {
        Self::Label(Label(val.to_string()))
    }
}
impl From<i64> for Operand {
    fn from(val: i64) -> Self {
        Self::Imm(Imm(val))
    }
}
impl From<f64> for Operand {
    fn from(val: f64) -> Self {
        Self::Fmm(Fmm(val))
    }
}
// impl Deref for Operand
impl Deref for Imm {
    type Target = i64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Deref for Fmm {
    type Target = f64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Deref for Label {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Deref for Reg {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

/// always zero register
pub const REG_ZERO: Reg = Reg::new(0, true);
/// return address register
pub const REG_RA: Reg = Reg::new(1, true);
/// stack pointer register
pub const REG_SP: Reg = Reg::new(2, true);
/// global pointer register
pub const REG_GP: Reg = Reg::new(3, true);
/// thread pointer register
pub const REG_TP: Reg = Reg::new(4, true);
/// temporary register
pub const REG_T0: Reg = Reg::new(5, true);
pub const REG_T1: Reg = Reg::new(6, true);
pub const REG_T2: Reg = Reg::new(7, true);

pub const REG_S0: Reg = Reg::new(8, true);
pub const REG_S1: Reg = Reg::new(9, true);
// argument register
pub const REG_A0: Reg = Reg::new(10, true);
pub const REG_A1: Reg = Reg::new(11, true);
pub const REG_A2: Reg = Reg::new(12, true);
pub const REG_A3: Reg = Reg::new(13, true);
pub const REG_A4: Reg = Reg::new(14, true);
pub const REG_A5: Reg = Reg::new(15, true);
pub const REG_A6: Reg = Reg::new(16, true);
pub const REG_A7: Reg = Reg::new(17, true);
pub const REG_S2: Reg = Reg::new(18, true);
pub const REG_S3: Reg = Reg::new(19, true);
pub const REG_S4: Reg = Reg::new(20, true);
pub const REG_S5: Reg = Reg::new(21, true);
pub const REG_S6: Reg = Reg::new(22, true);
pub const REG_S7: Reg = Reg::new(23, true);
pub const REG_S8: Reg = Reg::new(24, true);
pub const REG_S9: Reg = Reg::new(25, true);
pub const REG_S10: Reg = Reg::new(26, true);
pub const REG_S11: Reg = Reg::new(27, true);
pub const REG_T3: Reg = Reg::new(28, true);
pub const REG_T4: Reg = Reg::new(29, true);
pub const REG_T5: Reg = Reg::new(30, true);
pub const REG_T6: Reg = Reg::new(31, true);

// 浮点寄存器
pub const REG_FT0: Reg = Reg::new(0, false);
pub const REG_FT1: Reg = Reg::new(1, false);
pub const REG_FT2: Reg = Reg::new(2, false);
pub const REG_FT3: Reg = Reg::new(3, false);
pub const REG_FT4: Reg = Reg::new(4, false);
pub const REG_FT5: Reg = Reg::new(5, false);
pub const REG_FT6: Reg = Reg::new(6, false);
pub const REG_FT7: Reg = Reg::new(7, false);
pub const REG_FS0: Reg = Reg::new(8, false);
pub const REG_FS1: Reg = Reg::new(9, false);
pub const REG_FA0: Reg = Reg::new(10, false);
pub const REG_FA1: Reg = Reg::new(11, false);
pub const REG_FA2: Reg = Reg::new(12, false);
pub const REG_FA3: Reg = Reg::new(13, false);
pub const REG_FA4: Reg = Reg::new(14, false);
pub const REG_FA5: Reg = Reg::new(15, false);
pub const REG_FA6: Reg = Reg::new(16, false);
pub const REG_FA7: Reg = Reg::new(17, false);
pub const REG_FS2: Reg = Reg::new(18, false);
pub const REG_FS3: Reg = Reg::new(19, false);
pub const REG_FS4: Reg = Reg::new(20, false);
pub const REG_FS5: Reg = Reg::new(21, false);
pub const REG_FS6: Reg = Reg::new(22, false);
pub const REG_FS7: Reg = Reg::new(23, false);
pub const REG_FS8: Reg = Reg::new(24, false);
pub const REG_FS9: Reg = Reg::new(25, false);
pub const REG_FS10: Reg = Reg::new(26, false);
pub const REG_FS11: Reg = Reg::new(27, false);
pub const REG_FT8: Reg = Reg::new(28, false);
pub const REG_FT9: Reg = Reg::new(29, false);
pub const REG_FT10: Reg = Reg::new(30, false);
pub const REG_FT11: Reg = Reg::new(31, false);

static USUAL_REG_COUNTER: Lazy<ParalCounter> = Lazy::new(|| ParalCounter::new(32, 100_000_000));
static FLOAT_REG_COUNTER: Lazy<ParalCounter> = Lazy::new(|| ParalCounter::new(32, 100_000_000));
impl Reg {
    #[inline]
    pub fn gen_asm(&self) -> String {
        if self.is_phisic() {
            match self.is_usual {
                true => match self.id {
                    0 => String::from("zero"),
                    1 => String::from("ra"),
                    2 => String::from("sp"),
                    3 => String::from("gp"),
                    4 => String::from("tp"),
                    5 => String::from("t0"),
                    6 => String::from("t1"),
                    7 => String::from("t2"),
                    8 => String::from("s0"),
                    9 => String::from("s1"),
                    10 => String::from("a0"),
                    11 => String::from("a1"),
                    12 => String::from("a2"),
                    13 => String::from("a3"),
                    14 => String::from("a4"),
                    15 => String::from("a5"),
                    16 => String::from("a6"),
                    17 => String::from("a7"),
                    18 => String::from("s2"),
                    19 => String::from("s3"),
                    20 => String::from("s4"),
                    21 => String::from("s5"),
                    22 => String::from("s6"),
                    23 => String::from("s7"),
                    24 => String::from("s8"),
                    25 => String::from("s9"),
                    26 => String::from("s10"),
                    27 => String::from("s11"),
                    28 => String::from("t3"),
                    29 => String::from("t4"),
                    30 => String::from("t5"),
                    31 => String::from("t6"),
                    _ => panic!("Invalid register id"),
                },
                false => match self.id {
                    0 => String::from("ft0"),
                    1 => String::from("ft1"),
                    2 => String::from("ft2"),
                    3 => String::from("ft3"),
                    4 => String::from("ft4"),
                    5 => String::from("ft5"),
                    6 => String::from("ft6"),
                    7 => String::from("ft7"),
                    8 => String::from("fs0"),
                    9 => String::from("fs1"),
                    10 => String::from("fa0"),
                    11 => String::from("fa1"),
                    12 => String::from("fa2"),
                    13 => String::from("fa3"),
                    14 => String::from("fa4"),
                    15 => String::from("fa5"),
                    16 => String::from("fa6"),
                    17 => String::from("fa7"),
                    18 => String::from("fs2"),
                    19 => String::from("fs3"),
                    20 => String::from("fs4"),
                    21 => String::from("fs5"),
                    22 => String::from("fs6"),
                    23 => String::from("fs7"),
                    24 => String::from("fs8"),
                    25 => String::from("fs9"),
                    26 => String::from("fs10"),
                    27 => String::from("fs11"),
                    28 => String::from("ft8"),
                    29 => String::from("ft9"),
                    30 => String::from("ft10"),
                    31 => String::from("ft11"),
                    _ => panic!("Invalid register id"),
                },
            }
        } else {
            panic!("gen_asm for virtual reg is not implemented");
        }
    }

    #[inline]
    pub fn to_str(&self) -> String {
        match self.is_usual {
            true => format!("x{}", self.id),
            false => format!("f{}", self.id),
        }
    }

    #[inline]
    pub fn is_phisic(&self) -> bool {
        (0..=31).contains(&self.id)
    }
    #[inline]
    pub fn is_virtual(&self) -> bool {
        !self.is_phisic()
    }
    // 生成一个虚拟通用寄存器
    pub fn gen_virtual_usual_reg() -> Self {
        let id = USUAL_REG_COUNTER.get_id().unwrap();
        Self::new(id as u32, true)
    }
    // 生成一个虚拟浮点寄存器
    pub fn gen_virtual_float_reg() -> Self {
        let id = FLOAT_REG_COUNTER.get_id().unwrap();
        Self::new(id as u32, false)
    }
}
impl Imm {
    pub fn gen_asm(&self) -> String {
        format!("{}", self.0)
    }
}
impl Fmm {
    pub fn gen_asm(&self) -> String {
        format!("{}", self.0)
    }
}
impl Label {
    pub fn gen_asm(&self) -> String {
        self.0.clone()
    }
}

impl Operand {
    pub fn reg(&self) -> Option<Reg> {
        match self {
            Self::Reg(reg) => Some(reg.clone()),
            _ => None,
        }
    }
    pub fn imm(&self) -> Option<Imm> {
        match self {
            Self::Imm(imm) => Some(imm.clone()),
            _ => None,
        }
    }
    pub fn fmm(&self) -> Option<Fmm> {
        match self {
            Self::Fmm(fmm) => Some(fmm.clone()),
            _ => None,
        }
    }
    pub fn label(&self) -> Option<Label> {
        match self {
            Self::Label(label) => Some(label.clone()),
            _ => None,
        }
    }

    pub fn gen_asm(&self) -> String {
        match self {
            Self::Reg(reg) => reg.gen_asm(),
            Self::Imm(imm) => imm.gen_asm(),
            Self::Fmm(fmm) => fmm.gen_asm(),
            Self::Label(label) => label.gen_asm(),
        }
    }
}
