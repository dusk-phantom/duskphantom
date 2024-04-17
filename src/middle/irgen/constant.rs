use crate::errors::MiddelError;
use crate::frontend::Type;
use crate::middle::ir::{Constant, ValueType};
use crate::middle::irgen::util;
use std::cmp;
use std::ops;

/// Convert a type to its default constant
pub fn type_to_const(ty: &Type) -> Result<Vec<Constant>, MiddelError> {
    match ty {
        Type::Void => Err(MiddelError::GenError),
        Type::Int32 => Ok(vec![Constant::Int(0)]),
        Type::Float32 => Ok(vec![Constant::Float(0.0)]),
        Type::String => Err(MiddelError::GenError),
        Type::Char => Err(MiddelError::GenError),
        Type::Boolean => Ok(vec![Constant::Bool(false)]),
        Type::Pointer(_) => Err(MiddelError::GenError),
        Type::Array(ty, num) => Ok(util::repeat_vec(type_to_const(ty)?, *num)),
        Type::Function(_, _) => Err(MiddelError::GenError),
        Type::Enum(_) => Err(MiddelError::GenError),
        Type::Union(_) => Err(MiddelError::GenError),
        Type::Struct(_) => Err(MiddelError::GenError),
    }
}

/// Type cast for constant
impl From<Constant> for i32 {
    fn from(val: Constant) -> Self {
        match val {
            Constant::Int(x) => x,
            Constant::Float(x) => x as i32,
            Constant::Bool(x) => x as i32,
        }
    }
}

impl From<Constant> for f32 {
    fn from(val: Constant) -> Self {
        match val {
            Constant::Int(x) => x as f32,
            Constant::Float(x) => x,
            Constant::Bool(x) => x as i32 as f32,
        }
    }
}

impl From<Constant> for bool {
    fn from(val: Constant) -> Self {
        match val {
            Constant::Int(x) => x != 0,
            Constant::Float(x) => x != 0.0,
            Constant::Bool(x) => x,
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
        let max_ty = self.get_type().max_with(&rhs.get_type());
        match max_ty {
            ValueType::Float => (Into::<f32>::into(self) + Into::<f32>::into(rhs)).into(),
            ValueType::Int | ValueType::Bool => {
                (Into::<i32>::into(self) + Into::<i32>::into(rhs)).into()
            }
            _ => todo!(),
        }
    }
}

impl ops::Sub for Constant {
    type Output = Constant;

    fn sub(self, rhs: Constant) -> Self::Output {
        let max_ty = self.get_type().max_with(&rhs.get_type());
        match max_ty {
            ValueType::Float => (Into::<f32>::into(self) - Into::<f32>::into(rhs)).into(),
            ValueType::Int | ValueType::Bool => {
                (Into::<i32>::into(self) - Into::<i32>::into(rhs)).into()
            }
            _ => todo!(),
        }
    }
}

impl ops::Mul for Constant {
    type Output = Constant;

    fn mul(self, rhs: Constant) -> Self::Output {
        let max_ty = self.get_type().max_with(&rhs.get_type());
        match max_ty {
            ValueType::Float => (Into::<f32>::into(self) * Into::<f32>::into(rhs)).into(),
            ValueType::Int | ValueType::Bool => {
                (Into::<i32>::into(self) * Into::<i32>::into(rhs)).into()
            }
            _ => todo!(),
        }
    }
}

impl ops::Div for Constant {
    type Output = Constant;

    fn div(self, rhs: Constant) -> Self::Output {
        let max_ty = self.get_type().max_with(&rhs.get_type());
        match max_ty {
            ValueType::Float => (Into::<f32>::into(self) / Into::<f32>::into(rhs)).into(),
            ValueType::Int | ValueType::Bool => {
                (Into::<i32>::into(self) / Into::<i32>::into(rhs)).into()
            }
            _ => todo!(),
        }
    }
}

impl ops::Rem for Constant {
    type Output = Constant;

    fn rem(self, rhs: Constant) -> Self::Output {
        (Into::<i32>::into(self) % Into::<i32>::into(rhs)).into()
    }
}

impl cmp::PartialEq for Constant {
    fn eq(&self, other: &Constant) -> bool {
        let max_ty = self.get_type().max_with(&other.get_type());
        match max_ty {
            ValueType::Float => Into::<f32>::into(*self) == Into::<f32>::into(*other),
            ValueType::Int => Into::<i32>::into(*self) == Into::<i32>::into(*other),
            ValueType::Bool => Into::<bool>::into(*self) == Into::<bool>::into(*other),
            _ => todo!(),
        }
    }
}

impl cmp::PartialOrd for Constant {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let max_ty = self.get_type().max_with(&other.get_type());
        match max_ty {
            ValueType::Float => Into::<f32>::into(*self).partial_cmp(&Into::<f32>::into(*other)),
            ValueType::Int => Into::<i32>::into(*self).partial_cmp(&Into::<i32>::into(*other)),
            ValueType::Bool => Into::<bool>::into(*self).partial_cmp(&Into::<bool>::into(*other)),
            _ => todo!(),
        }
    }
}
