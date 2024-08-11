use std::{
    cmp,
    hash::{Hash, Hasher},
    ops,
};

use instruction::InstType;

use super::*;

#[derive(Clone, Debug)]
pub enum Constant {
    SignedChar(i8),
    Int(i32),
    Float(f32),
    Bool(bool),
    Array(Vec<Constant>),
    Zero(ValueType),
}

impl PartialEq for Constant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Constant::SignedChar(c1), Constant::SignedChar(c2)) => c1 == c2,
            (Constant::Int(i1), Constant::Int(i2)) => i1 == i2,
            // Compare float in bits to have `Eq` trait implemented
            (Constant::Float(f1), Constant::Float(f2)) => f1.to_bits() == f2.to_bits(),
            (Constant::Bool(b1), Constant::Bool(b2)) => b1 == b2,
            (Constant::Array(arr1), Constant::Array(arr2)) => arr1 == arr2,
            (Constant::Zero(t1), Constant::Zero(t2)) => t1 == t2,
            _ => false,
        }
    }
}

impl Eq for Constant {}

impl Hash for Constant {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Constant::SignedChar(c) => c.hash(state),
            Constant::Int(i) => i.hash(state),
            Constant::Float(f) => f.to_bits().hash(state),
            Constant::Bool(b) => b.hash(state),
            Constant::Array(arr) => arr.hash(state),
            Constant::Zero(t) => t.hash(state),
        }
    }
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::SignedChar(c) => write!(f, "{}", c),
            Constant::Int(i) => write!(f, "{}", i),
            Constant::Float(fl) => {
                // write float in hexidemal form (IEEE-754) like 0x1234567800000000
                let bytes = (*fl as f64).to_le_bytes();
                write!(f, "0x")?;
                for byte in bytes.iter().rev() {
                    write!(f, "{:02x}", byte)?;
                }
                Ok(())
            }
            Constant::Bool(b) => write!(f, "{}", b),
            Constant::Array(arr) => {
                write!(f, "[")?;
                for (i, c) in arr.iter().enumerate() {
                    write!(f, "{} {}", c.get_type(), c)?;
                    if i != arr.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            Constant::Zero(_) => write!(f, "zeroinitializer"),
        }
    }
}

impl Constant {
    pub fn get_type(&self) -> ValueType {
        match self {
            Constant::SignedChar(_) => ValueType::SignedChar,
            Constant::Int(_) => ValueType::Int,
            Constant::Float(_) => ValueType::Float,
            Constant::Bool(_) => ValueType::Bool,
            Constant::Array(arr) => {
                let sub_type = arr.first().unwrap().get_type();
                ValueType::Array(Box::new(sub_type), arr.len())
            }
            Constant::Zero(t) => t.clone(),
        }
    }

    /// Apply instruction on identity and this constant.
    /// For example, apply(4, Sub) = 0 - 4 = 4.
    ///
    /// # Panics
    /// Please make sure constant type and inst type is supported.
    pub fn apply(self, ty: InstType) -> Constant {
        match ty {
            InstType::Add | InstType::FAdd => self,
            InstType::Sub | InstType::FSub => -self,
            _ => unimplemented!(),
        }
    }
}

impl From<i32> for Constant {
    fn from(i: i32) -> Self {
        Self::Int(i)
    }
}

impl From<u32> for Constant {
    fn from(u: u32) -> Self {
        Self::Int(u as i32)
    }
}

impl From<f32> for Constant {
    fn from(fl: f32) -> Self {
        Self::Float(fl)
    }
}

impl From<bool> for Constant {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<Constant> for i32 {
    fn from(val: Constant) -> Self {
        match val {
            Constant::Int(x) => x,
            Constant::Float(x) => x as i32,
            Constant::Bool(x) => x as i32,
            _ => panic!("Cannot cast {} to i32", val),
        }
    }
}

impl From<Constant> for u32 {
    fn from(val: Constant) -> Self {
        match val {
            Constant::Int(x) => x as u32,
            Constant::Float(x) => x as u32,
            Constant::Bool(x) => x as u32,
            _ => panic!("Cannot cast {} to u32", val),
        }
    }
}

impl From<Constant> for f32 {
    fn from(val: Constant) -> Self {
        match val {
            Constant::Int(x) => x as f32,
            Constant::Float(x) => x,
            Constant::Bool(x) => x as i32 as f32,
            _ => panic!("Cannot cast {} to f32", val),
        }
    }
}

impl From<Constant> for bool {
    fn from(val: Constant) -> Self {
        match val {
            Constant::Int(x) => x != 0,
            Constant::Float(x) => x != 0.0,
            Constant::Bool(x) => x,
            _ => panic!("Cannot cast {} to bool", val),
        }
    }
}

impl Constant {
    pub fn cast(self, ty: &ValueType) -> Self {
        match ty {
            ValueType::Int => Into::<i32>::into(self).into(),
            ValueType::Float => Into::<f32>::into(self).into(),
            ValueType::Bool => Into::<bool>::into(self).into(),
            ValueType::Array(element_ty, _) => {
                let arr = match self {
                    Constant::Array(arr) => arr,
                    _ => panic!("Cannot convert {} to array", self),
                };
                Constant::Array(arr.into_iter().map(|x| x.cast(element_ty)).collect())
            }
            _ => self,
        }
    }
}

/// Override operators for constant
impl ops::Neg for Constant {
    type Output = Constant;

    fn neg(self) -> Self::Output {
        let ty = self.get_type();
        match ty {
            ValueType::Float => (-Into::<f32>::into(self)).into(),
            ValueType::Int | ValueType::Bool => (-Into::<i32>::into(self)).into(),
            _ => todo!(),
        }
    }
}

impl ops::Not for Constant {
    type Output = Constant;

    fn not(self) -> Self::Output {
        (!Into::<bool>::into(self)).into()
    }
}

impl ops::Add for Constant {
    type Output = Constant;

    fn add(self, rhs: Constant) -> Self::Output {
        let ty = self.get_type();
        match ty {
            ValueType::Float => (Into::<f32>::into(self) + Into::<f32>::into(rhs)).into(),
            ValueType::Int | ValueType::Bool => {
                (Into::<i32>::into(self).wrapping_add(Into::<i32>::into(rhs))).into()
            }
            _ => todo!(),
        }
    }
}

impl ops::Sub for Constant {
    type Output = Constant;

    fn sub(self, rhs: Constant) -> Self::Output {
        let ty = self.get_type();
        match ty {
            ValueType::Float => (Into::<f32>::into(self) - Into::<f32>::into(rhs)).into(),
            ValueType::Int | ValueType::Bool => {
                (Into::<i32>::into(self).wrapping_sub(Into::<i32>::into(rhs))).into()
            }
            _ => todo!(),
        }
    }
}

impl ops::Mul for Constant {
    type Output = Constant;

    fn mul(self, rhs: Constant) -> Self::Output {
        let ty = self.get_type();
        match ty {
            ValueType::Float => (Into::<f32>::into(self) * Into::<f32>::into(rhs)).into(),
            ValueType::Int | ValueType::Bool => {
                (Into::<i32>::into(self).wrapping_mul(Into::<i32>::into(rhs))).into()
            }
            _ => todo!(),
        }
    }
}

impl ops::Div for Constant {
    type Output = Constant;

    fn div(self, rhs: Constant) -> Self::Output {
        let ty = self.get_type();
        match ty {
            ValueType::Float => (Into::<f32>::into(self) / Into::<f32>::into(rhs)).into(),
            ValueType::Int | ValueType::Bool => {
                (Into::<i32>::into(self).wrapping_div(Into::<i32>::into(rhs))).into()
            }
            _ => todo!(),
        }
    }
}

impl ops::Rem for Constant {
    type Output = Constant;

    fn rem(self, rhs: Constant) -> Self::Output {
        (Into::<i32>::into(self).wrapping_rem(Into::<i32>::into(rhs))).into()
    }
}

impl ops::Shl for Constant {
    type Output = Constant;

    fn shl(self, rhs: Constant) -> Self::Output {
        (Into::<i32>::into(self).wrapping_shl(Into::<i32>::into(rhs) as u32)).into()
    }
}

impl ops::Shr for Constant {
    type Output = Constant;

    fn shr(self, rhs: Constant) -> Self::Output {
        (Into::<i32>::into(self).wrapping_shr(Into::<i32>::into(rhs) as u32)).into()
    }
}

impl ops::BitAnd for Constant {
    type Output = Constant;

    fn bitand(self, rhs: Constant) -> Self::Output {
        (Into::<i32>::into(self) & Into::<i32>::into(rhs)).into()
    }
}

impl ops::BitOr for Constant {
    type Output = Constant;

    fn bitor(self, rhs: Constant) -> Self::Output {
        (Into::<i32>::into(self) | Into::<i32>::into(rhs)).into()
    }
}

impl ops::BitXor for Constant {
    type Output = Constant;

    fn bitxor(self, rhs: Constant) -> Self::Output {
        (Into::<i32>::into(self) ^ Into::<i32>::into(rhs)).into()
    }
}

impl cmp::PartialOrd for Constant {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let ty = self.get_type();
        match ty {
            ValueType::Float => {
                Into::<f32>::into(self.clone()).partial_cmp(&Into::<f32>::into(other.clone()))
            }
            ValueType::Int => {
                Into::<i32>::into(self.clone()).partial_cmp(&Into::<i32>::into(other.clone()))
            }
            ValueType::Bool => {
                Into::<bool>::into(self.clone()).partial_cmp(&Into::<bool>::into(other.clone()))
            }
            _ => todo!(),
        }
    }
}
