use crate::errors::MiddleError;
use crate::frontend::Type;
use crate::middle::ir::{Constant, ValueType};
use std::cmp;
use std::ops;

/// Convert a type to its default constant
pub fn type_to_const(ty: &Type) -> Result<Constant, MiddleError> {
    match ty {
        Type::Void => Err(MiddleError::GenError),
        Type::Int32 => Ok(Constant::Int(0)),
        Type::Float32 => Ok(Constant::Float(0.0)),
        Type::String => Err(MiddleError::GenError),
        Type::Char => Err(MiddleError::GenError),
        Type::Boolean => Ok(Constant::Bool(false)),
        Type::Pointer(_) => Err(MiddleError::GenError),
        Type::Array(ty, num) => {
            let inner_const = type_to_const(ty)?;
            Ok(Constant::Array(vec![inner_const; *num]))
        }
        Type::Function(_, _) => Err(MiddleError::GenError),
        Type::Enum(_) => Err(MiddleError::GenError),
        Type::Union(_) => Err(MiddleError::GenError),
        Type::Struct(_) => Err(MiddleError::GenError),
    }
}

/// Type cast for constant
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
        match (self, other) {
            // For array, compare length and all elements
            (Constant::Array(a), Constant::Array(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for (x, y) in a.iter().zip(b.iter()) {
                    if x != y {
                        return false;
                    }
                }
                true
            }

            // If only one of them is array, they are not equal
            (Constant::Array(_), _) | (_, Constant::Array(_)) => false,

            // For other types, cast to maximum type and then compare
            (a, b) => {
                let max_ty = self.get_type().max_with(&other.get_type());
                match max_ty {
                    ValueType::Float => {
                        Into::<f32>::into(a.clone()) == Into::<f32>::into(b.clone())
                    }
                    ValueType::Int => Into::<i32>::into(a.clone()) == Into::<i32>::into(b.clone()),
                    ValueType::Bool => {
                        Into::<bool>::into(a.clone()) == Into::<bool>::into(b.clone())
                    }
                    _ => todo!(),
                }
            }
        }
    }
}

impl cmp::PartialOrd for Constant {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let max_ty = self.get_type().max_with(&other.get_type());
        match max_ty {
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
