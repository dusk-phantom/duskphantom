use crate::utils::paral_counter::ParalCounter;
use std::hash::Hash;
use std::ops::Deref;

use super::{BackendError, StackSlot};

#[derive(Clone, Debug)]
pub enum Operand {
    Reg(Reg),
    Imm(Imm),
    Fmm(Fmm),
    StackSlot(StackSlot),
    Label(Label),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reg {
    id: u32,
    is_usual: bool,
}
impl Reg {
    #[inline]
    pub const fn caller_save_regs() -> &'static [Reg; 35] {
        &[
            REG_T0, REG_T1, REG_T2, REG_T3, REG_T4, REG_T5, REG_T6, REG_A0, REG_A1, REG_A2, REG_A3,
            REG_A4, REG_A5, REG_A6, REG_A7, REG_FT0, REG_FT1, REG_FT2, REG_FT3, REG_FT4, REG_FT5,
            REG_FT6, REG_FT7, REG_FT8, REG_FT9, REG_FT10, REG_FT11, REG_FA0, REG_FA1, REG_FA2,
            REG_FA3, REG_FA4, REG_FA5, REG_FA6, REG_FA7,
        ]
    }

    #[inline]
    pub const fn callee_save_regs() -> &'static [Reg; 23] {
        &[
            REG_S1, REG_S2, REG_S3, REG_S4, REG_S5, REG_S6, REG_S7, REG_S8, REG_S9, REG_S10,
            REG_S11, REG_FS0, REG_FS1, REG_FS2, REG_FS3, REG_FS4, REG_FS5, REG_FS6, REG_FS7,
            REG_FS8, REG_FS9, REG_FS10, REG_FS11,
        ]
    }

    pub const fn physical_regs() -> &'static [Reg; 64] {
        &[
            // usual registers
            REG_ZERO, REG_RA, REG_SP, REG_GP, REG_TP, REG_T0, REG_T1, REG_T2, REG_S0, REG_S1,
            REG_A0, REG_A1, REG_A2, REG_A3, REG_A4, REG_A5, REG_A6, REG_A7, REG_S2, REG_S3, REG_S4,
            REG_S5, REG_S6, REG_S7, REG_S8, REG_S9, REG_S10, REG_S11, REG_T3, REG_T4, REG_T5,
            REG_T6, // float registers
            REG_FT0, REG_FT1, REG_FT2, REG_FT3, REG_FT4, REG_FT5, REG_FT6, REG_FT7, REG_FS0,
            REG_FS1, REG_FA0, REG_FA1, REG_FA2, REG_FA3, REG_FA4, REG_FA5, REG_FA6, REG_FA7,
            REG_FS2, REG_FS3, REG_FS4, REG_FS5, REG_FS6, REG_FS7, REG_FS8, REG_FS9, REG_FS10,
            REG_FS11, REG_FT8, REG_FT9, REG_FT10, REG_FT11,
        ]
    }

    pub const fn new(id: u32, is_usual: bool) -> Self {
        Self { id, is_usual }
    }
    pub const fn new_usual(id: u32) -> Self {
        Self::new(id, true)
    }
    pub const fn new_float(id: u32) -> Self {
        Self::new(id, false)
    }

    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }
    #[inline]
    pub fn is_usual(&self) -> bool {
        self.is_usual
    }
    #[inline]
    pub fn is_float(&self) -> bool {
        !self.is_usual
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Imm(i64);

impl TryInto<u32> for Imm {
    type Error = BackendError;
    fn try_into(self) -> Result<u32, Self::Error> {
        if self.0 >= 0 && self.0 <= u32::MAX as i64 {
            Ok(self.0 as u32)
        } else {
            Err(BackendError::InternalConsistencyError(
                "Imm is not a valid u32".to_string(),
            ))
        }
    }
}

#[derive(Clone, Debug)]
pub struct Fmm(f64);

impl TryInto<f32> for Fmm {
    type Error = BackendError;
    fn try_into(self) -> Result<f32, Self::Error> {
        (&self).try_into()
    }
}

impl TryInto<f32> for &Fmm {
    type Error = BackendError;
    fn try_into(self) -> Result<f32, Self::Error> {
        if f64::is_nan(self.0) {
            Ok(f32::NAN)
        } else if self.0 < f32::MAX as f64 && self.0 > f32::MIN as f64 {
            Ok(self.0 as f32)
        } else {
            Err(BackendError::InternalConsistencyError(
                "Fmm is not a valid f32".to_string(),
            ))
        }
    }
}

impl From<Imm> for i64 {
    fn from(value: Imm) -> Self {
        value.0
    }
}

impl std::ops::Neg for Imm {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl std::ops::Shl<u32> for Imm {
    type Output = Self;
    fn shl(self, rhs: u32) -> Self::Output {
        Self(self.0 << rhs)
    }
}
impl std::ops::Add for Imm {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Mul for Imm {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl PartialEq for Fmm {
    fn eq(&self, other: &Self) -> bool {
        if f64::is_nan(self.0) && f64::is_nan(other.0) {
            true
        } else {
            self.0 == other.0
        }
    }
}
impl Eq for Fmm {}
impl Hash for Fmm {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if f64::is_nan(self.0) {
            // NOTICE: f64::NAN.to_bits() is not always the same value in different platforms
            f64::NAN.to_bits().hash(state);
        } else {
            self.0.to_bits().hash(state);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Label(String);

// impl from for Operand
impl From<u32> for Imm {
    fn from(value: u32) -> Self {
        Self(value as i64)
    }
}
impl From<i32> for Imm {
    fn from(value: i32) -> Self {
        Self(value as i64)
    }
}
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
impl From<&f32> for Fmm {
    fn from(value: &f32) -> Self {
        if f32::is_nan(*value) {
            Self(f64::NAN)
        } else {
            Self(*value as f64)
        }
    }
}
impl From<&f64> for Fmm {
    fn from(value: &f64) -> Self {
        Self(*value)
    }
}

impl From<String> for Label {
    fn from(val: String) -> Self {
        Self(val)
    }
}

impl From<&String> for Label {
    fn from(val: &String) -> Self {
        Self(val.clone())
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
impl From<&Imm> for Operand {
    fn from(val: &Imm) -> Self {
        Self::Imm(val.clone())
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
impl From<String> for Operand {
    fn from(value: String) -> Self {
        Self::Label(Label(value))
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

// other registers
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

// float registers for temporary
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
/* float argument registers */
// used for return value
pub const REG_FA0: Reg = Reg::new(10, false);
pub const REG_FA1: Reg = Reg::new(11, false);
pub const REG_FA2: Reg = Reg::new(12, false);
pub const REG_FA3: Reg = Reg::new(13, false);
pub const REG_FA4: Reg = Reg::new(14, false);
pub const REG_FA5: Reg = Reg::new(15, false);
pub const REG_FA6: Reg = Reg::new(16, false);
pub const REG_FA7: Reg = Reg::new(17, false);
// other registers
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

#[derive(Debug, Clone)]
pub struct RegGenerator {
    usual_counter: ParalCounter,
    float_counter: ParalCounter,
}
impl Default for RegGenerator {
    fn default() -> Self {
        Self {
            usual_counter: ParalCounter::new(32, 100_000_000),
            float_counter: ParalCounter::new(32, 100_000_000),
        }
    }
}
impl RegGenerator {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn gen_virtual_reg(&mut self, is_usual: bool) -> Reg {
        if is_usual {
            self.gen_virtual_usual_reg()
        } else {
            self.gen_virtual_float_reg()
        }
    }
    pub fn gen_virtual_usual_reg(&mut self) -> Reg {
        let id = self.usual_counter.get_id().unwrap();
        Reg::new(id as u32, true)
    }
    pub fn gen_virtual_float_reg(&mut self) -> Reg {
        let id = self.float_counter.get_id().unwrap();
        Reg::new(id as u32, false)
    }
}

impl Reg {
    #[inline]
    pub fn gen_asm(&self) -> String {
        if self.is_physical() {
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
            #[cfg(feature = "gen_virtual_asm")]
            {
                if self.is_usual {
                    format!("x{}", self.id)
                } else {
                    format!("f{}", self.id)
                }
            }
            #[cfg(not(feature = "gen_virtual_asm"))]
            {
                panic!("gen_asm for virtual reg is not implemented");
            }
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
    pub fn is_physical(&self) -> bool {
        (0..=31).contains(&self.id)
    }

    #[inline]
    pub fn is_virtual(&self) -> bool {
        !self.is_physical()
    }

    #[inline]
    pub fn is_caller_save(&self) -> bool {
        Self::caller_save_regs().contains(self)
    }

    #[inline]
    pub fn is_callee_save(&self) -> bool {
        Self::callee_save_regs().contains(self)
    }
}
impl Imm {
    #[inline]
    pub fn gen_asm(&self) -> String {
        format!("{}", self.0)
    }

    pub fn in_limit(&self, bits: usize) -> bool {
        if (1..=64).contains(&bits) {
            let max = (1 << (bits - 1)) - 1;
            let min = -(1 << (bits - 1));
            self.0 <= max && self.0 >= min
        } else {
            // if bits is 0, then no limit; else bits is greater than 64, then return false
            bits != 0
        }
    }

    #[inline]
    pub fn in_limit_12(&self) -> bool {
        self.in_limit(12)
    }
}

impl Fmm {
    #[inline]
    pub fn gen_asm(&self) -> String {
        format!("{}", self.0)
    }
}
impl Label {
    #[inline]
    pub fn gen_asm(&self) -> String {
        self.0.clone()
    }
}

impl Operand {
    #[inline]
    pub fn reg(&self) -> Option<Reg> {
        match self {
            Self::Reg(reg) => Some(*reg),
            _ => None,
        }
    }
    #[inline]
    pub fn imm(&self) -> Option<Imm> {
        match self {
            Self::Imm(imm) => Some(imm.clone()),
            _ => None,
        }
    }
    #[inline]
    pub fn fmm(&self) -> Option<Fmm> {
        match self {
            Self::Fmm(fmm) => Some(fmm.clone()),
            _ => None,
        }
    }
    #[inline]
    pub fn label(&self) -> Option<Label> {
        match self {
            Self::Label(label) => Some(label.clone()),
            _ => None,
        }
    }
    #[inline]
    pub fn gen_asm(&self) -> String {
        match self {
            Self::Reg(reg) => reg.gen_asm(),
            Self::Imm(imm) => imm.gen_asm(),
            Self::Fmm(fmm) => fmm.gen_asm(),
            Self::StackSlot(stack_slot) => stack_slot.gen_asm(),
            Self::Label(label) => label.gen_asm(),
        }
    }
}

impl TryInto<Reg> for Operand {
    type Error = BackendError;
    fn try_into(self) -> Result<Reg, Self::Error> {
        match self {
            Operand::Reg(reg) => Ok(reg),
            _ => Err(BackendError::InternalConsistencyError(
                "Operand is not a Reg".to_string(),
            )),
        }
    }
}
impl TryInto<Imm> for usize {
    type Error = BackendError;
    fn try_into(self) -> Result<Imm, Self::Error> {
        if self > i64::MAX as usize {
            Err(BackendError::InternalConsistencyError(
                "usize is too large to convert to Imm".to_string(),
            ))
        } else {
            Ok(Imm(self as i64))
        }
    }
}
impl TryInto<usize> for Imm {
    type Error = BackendError;
    fn try_into(self) -> Result<usize, Self::Error> {
        if (i64::MAX as u128) < (usize::MAX as u128) && self.0 >= 0 {
            Ok(self.0 as usize)
        } else {
            Err(BackendError::InternalConsistencyError(
                "Imm is invalid  to convert to usize".to_string(),
            ))
        }
    }
}

impl TryInto<Imm> for Operand {
    type Error = BackendError;
    fn try_into(self) -> Result<Imm, Self::Error> {
        match self {
            Operand::Imm(imm) => Ok(imm),
            _ => Err(BackendError::InternalConsistencyError(
                "Operand is not a Imm".to_string(),
            )),
        }
    }
}

impl TryInto<Imm> for &Operand {
    type Error = BackendError;
    fn try_into(self) -> Result<Imm, Self::Error> {
        match self {
            Operand::Imm(imm) => Ok(imm.clone()),
            _ => Err(BackendError::InternalConsistencyError(
                "Operand is not a Imm".to_string(),
            )),
        }
    }
}

impl TryInto<Fmm> for Operand {
    type Error = BackendError;
    fn try_into(self) -> Result<Fmm, Self::Error> {
        match self {
            Operand::Fmm(fmm) => Ok(fmm),
            _ => Err(BackendError::InternalConsistencyError(
                "Operand is not a Fmm".to_string(),
            )),
        }
    }
}
impl TryInto<StackSlot> for Operand {
    type Error = BackendError;
    fn try_into(self) -> Result<StackSlot, Self::Error> {
        match self {
            Operand::StackSlot(stack_slot) => Ok(stack_slot),
            _ => Err(BackendError::InternalConsistencyError(
                "Operand is not a StackSlot".to_string(),
            )),
        }
    }
}

impl TryInto<Label> for Operand {
    type Error = BackendError;
    fn try_into(self) -> Result<Label, Self::Error> {
        match self {
            Operand::Label(label) => Ok(label),
            _ => Err(BackendError::InternalConsistencyError(
                "Operand is not a Label".to_string(),
            )),
        }
    }
}

impl<'a> TryInto<&'a Label> for &'a Operand {
    type Error = BackendError;
    fn try_into(self) -> Result<&'a Label, Self::Error> {
        match self {
            Operand::Label(label) => Ok(label),
            _ => Err(BackendError::InternalConsistencyError(
                "Operand is not a Label".to_string(),
            )),
        }
    }
}
impl<'a> TryInto<&'a str> for &'a Label {
    type Error = BackendError;
    fn try_into(self) -> Result<&'a str, Self::Error> {
        Ok(self.0.as_str())
    }
}
impl From<&Reg> for Operand {
    fn from(value: &Reg) -> Self {
        Operand::Reg(*value)
    }
}
impl From<&StackSlot> for Operand {
    fn from(value: &StackSlot) -> Self {
        Operand::StackSlot(*value)
    }
}

/// 单元测试
#[cfg(test)]
pub mod tests {
    use std::{
        collections::HashSet,
        sync::{Arc, Mutex},
    };

    use super::*;

    #[test]
    fn test_gen_reg() {
        let mut regs: HashSet<Reg> = HashSet::new();
        let reg_gener = RegGenerator::new();
        let reg_gener = Arc::new(Mutex::new(reg_gener));
        let mut handlers = vec![];
        for _ in 0..10 {
            let reg_gener = Arc::clone(&reg_gener);
            let handler = std::thread::spawn(move || {
                let mut regs = HashSet::new();
                let mut reg_gener = reg_gener.lock().unwrap();
                for _ in 0..1000 {
                    let reg = reg_gener.gen_virtual_usual_reg();
                    regs.insert(reg);
                    let reg = reg_gener.gen_virtual_float_reg();
                    regs.insert(reg);
                }
                regs
            });
            handlers.push(handler);
        }
        for handler in handlers {
            let par_regs = handler.join().unwrap();
            regs.extend(par_regs.iter().cloned())
        }
        assert_eq!(regs.len(), 20000);
        for reg in regs.iter() {
            assert!(reg.is_virtual());
        }
        for i in 0..10000 {
            let reg = Reg::new(i + 32, true);
            assert!(regs.contains(&reg));
            let reg = Reg::new(i + 32, false);
            assert!(regs.contains(&reg));
        }
    }

    #[test]
    fn test_special_reg() {
        let reg = Reg::new(0, true);
        assert_eq!(reg.gen_asm(), "zero");
        let reg = Reg::new(1, true);
        assert_eq!(reg.gen_asm(), "ra");
        let reg = Reg::new(2, true);
        assert_eq!(reg.gen_asm(), "sp");
        let reg = Reg::new(3, true);
        assert_eq!(reg.gen_asm(), "gp");
        let reg = Reg::new(4, true);
        assert_eq!(reg.gen_asm(), "tp");
    }
    #[test]
    fn test_usual_float() {
        for i in 0..=31 {
            let reg = Reg::new(i, true);
            assert_eq!(reg.to_str(), format!("x{}", i));
            assert!(reg.is_usual());
            let reg = Reg::new(i, false);
            assert_eq!(reg.to_str(), format!("f{}", i));
            assert!(!reg.is_usual());
        }
    }
    #[test]
    fn test_phisic_virtual() {
        for i in 0..=31 {
            let reg = Reg::new(i, true);
            assert!(reg.is_physical());
            assert!(!reg.is_virtual());
            let reg = Reg::new(i, false);
            assert!(reg.is_physical());
            assert!(!reg.is_virtual());
        }
        for i in 32..=127 {
            let reg = Reg::new(i, false);
            assert!(!reg.is_physical());
            assert!(reg.is_virtual());
        }
    }
    #[test]
    pub fn test_const_phisic_reg() {
        assert_eq!(REG_ZERO.gen_asm(), "zero");
        assert_eq!(REG_RA.gen_asm(), "ra");
        assert_eq!(REG_SP.gen_asm(), "sp");
        assert_eq!(REG_GP.gen_asm(), "gp");
        assert_eq!(REG_TP.gen_asm(), "tp");
        assert_eq!(REG_T0.gen_asm(), "t0");
        assert_eq!(REG_T1.gen_asm(), "t1");
        assert_eq!(REG_T2.gen_asm(), "t2");
        assert_eq!(REG_S0.gen_asm(), "s0");
        assert_eq!(REG_S1.gen_asm(), "s1");
        assert_eq!(REG_A0.gen_asm(), "a0");
        assert_eq!(REG_A1.gen_asm(), "a1");
        assert_eq!(REG_A2.gen_asm(), "a2");
        assert_eq!(REG_A3.gen_asm(), "a3");
        assert_eq!(REG_A4.gen_asm(), "a4");
        assert_eq!(REG_A5.gen_asm(), "a5");
        assert_eq!(REG_A6.gen_asm(), "a6");
        assert_eq!(REG_A7.gen_asm(), "a7");
        assert_eq!(REG_S2.gen_asm(), "s2");
        assert_eq!(REG_S3.gen_asm(), "s3");
        assert_eq!(REG_S4.gen_asm(), "s4");
        assert_eq!(REG_S5.gen_asm(), "s5");
        assert_eq!(REG_S6.gen_asm(), "s6");
        assert_eq!(REG_S7.gen_asm(), "s7");
        assert_eq!(REG_S8.gen_asm(), "s8");
        assert_eq!(REG_S9.gen_asm(), "s9");
        assert_eq!(REG_S10.gen_asm(), "s10");
        assert_eq!(REG_S11.gen_asm(), "s11");
        assert_eq!(REG_T3.gen_asm(), "t3");
        assert_eq!(REG_T4.gen_asm(), "t4");
        assert_eq!(REG_T5.gen_asm(), "t5");
        assert_eq!(REG_T6.gen_asm(), "t6");
        assert_eq!(REG_FT0.gen_asm(), "ft0");
        assert_eq!(REG_FT1.gen_asm(), "ft1");
        assert_eq!(REG_FT2.gen_asm(), "ft2");
        assert_eq!(REG_FT3.gen_asm(), "ft3");
        assert_eq!(REG_FT4.gen_asm(), "ft4");
        assert_eq!(REG_FT5.gen_asm(), "ft5");
        assert_eq!(REG_FT6.gen_asm(), "ft6");
        assert_eq!(REG_FT7.gen_asm(), "ft7");
        assert_eq!(REG_FS0.gen_asm(), "fs0");
        assert_eq!(REG_FS1.gen_asm(), "fs1");
        assert_eq!(REG_FA0.gen_asm(), "fa0");
        assert_eq!(REG_FA1.gen_asm(), "fa1");
        assert_eq!(REG_FA2.gen_asm(), "fa2");
        assert_eq!(REG_FA3.gen_asm(), "fa3");
        assert_eq!(REG_FA4.gen_asm(), "fa4");
        assert_eq!(REG_FA5.gen_asm(), "fa5");
        assert_eq!(REG_FA6.gen_asm(), "fa6");
        assert_eq!(REG_FA7.gen_asm(), "fa7");
        assert_eq!(REG_FS2.gen_asm(), "fs2");
        assert_eq!(REG_FS3.gen_asm(), "fs3");
        assert_eq!(REG_FS4.gen_asm(), "fs4");
        assert_eq!(REG_FS5.gen_asm(), "fs5");
        assert_eq!(REG_FS6.gen_asm(), "fs6");
        assert_eq!(REG_FS7.gen_asm(), "fs7");
        assert_eq!(REG_FS8.gen_asm(), "fs8");
        assert_eq!(REG_FS9.gen_asm(), "fs9");
        assert_eq!(REG_FS10.gen_asm(), "fs10");
        assert_eq!(REG_FS11.gen_asm(), "fs11");
        assert_eq!(REG_FT8.gen_asm(), "ft8");
        assert_eq!(REG_FT9.gen_asm(), "ft9");
        assert_eq!(REG_FT10.gen_asm(), "ft10");
        assert_eq!(REG_FT11.gen_asm(), "ft11");
    }

    #[test]
    fn test_f64_as_f32() {
        // FIXME
        let f1: f32 = f32::MIN;
        let f2: f64 = f1 as f64;
        let f3 = f2 as f32;
        assert!(f1 == f3);
    }
    #[test]
    fn test_constant_physical_regs() {
        let p_regs: HashSet<Reg> = Reg::physical_regs().iter().cloned().collect();
        assert_eq!(p_regs.len(), 64);
    }
}
