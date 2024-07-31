use anyhow::{anyhow, Context, Result};

use crate::context;

use super::Constant;

/// Represent the type of a value.
#[derive(Clone, PartialEq, Eq)]
pub enum ValueType {
    Void,
    SignedChar,
    Int,
    Float,
    Bool,
    Array(Box<ValueType>, usize),
    Pointer(Box<ValueType>),
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Void => write!(f, "void"),
            ValueType::SignedChar => write!(f, "i8"),
            ValueType::Int => write!(f, "i32"),
            ValueType::Float => write!(f, "float"),
            ValueType::Bool => write!(f, "i1"),
            ValueType::Array(one_type, size) => write!(f, "[{} x {}]", size, one_type),
            ValueType::Pointer(pointer) => write!(f, "{}*", pointer),
        }
    }
}

impl std::fmt::Debug for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl ValueType {
    pub fn is_basic_type(&self) -> bool {
        matches!(
            self,
            ValueType::Void | ValueType::Int | ValueType::Float | ValueType::Bool
        )
    }

    pub fn is_num(&self) -> bool {
        matches!(self, ValueType::Bool | ValueType::Int | ValueType::Float)
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self, ValueType::Pointer(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, ValueType::Array(_, _))
    }

    /// Get size of this value type.
    pub fn size(&self) -> usize {
        match self {
            ValueType::Array(element_type, dim) => *dim * element_type.size(),
            _ => 1,
        }
    }

    /// Get subtype of the value type.
    /// Subtype is the type of the element in the array or the type of the pointer.
    pub fn get_sub_type(&self) -> Option<&ValueType> {
        match self {
            ValueType::Array(sub_type, _) => Some(sub_type.as_ref()),
            ValueType::Pointer(sub_type) => Some(sub_type.as_ref()),
            _ => None,
        }
    }

    /// Get base type of the value type.
    /// Base type is i32 / f32 for array.
    pub fn get_base_type(&self) -> ValueType {
        match self {
            ValueType::Array(sub_type, _) => sub_type.get_base_type(),
            _ => self.clone(),
        }
    }

    /// Get default initializer of this type.
    pub fn default_initializer(&self) -> Result<Constant> {
        match self {
            ValueType::Void => {
                Err(anyhow!("Cannot convert void type to constant")).with_context(|| context!())
            }
            ValueType::Int => Ok(Constant::Int(0)),
            ValueType::SignedChar => Ok(Constant::SignedChar(0)),
            ValueType::Float => Ok(Constant::Float(0.0)),
            ValueType::Bool => Ok(Constant::Bool(false)),
            ValueType::Pointer(_) => {
                Err(anyhow!("Cannot convert pointer type to constant")).with_context(|| context!())
            }
            ValueType::Array(ty, _) => Ok(Constant::Zero(*ty.clone())),
        }
    }

    /// Convert a numeric value type to its precision level.
    /// Higher is more precise.
    pub fn to_precision_level(&self) -> i32 {
        match self {
            // All boolean should be converted to int when applying `+` and etc.
            ValueType::Bool => 1,
            ValueType::Int => 1,
            ValueType::Float => 2,
            _ => 0,
        }
    }

    /// Convert a precision level to a value type.
    pub fn from_precision_level(level: i32) -> Self {
        match level {
            1 => ValueType::Int,
            2 => ValueType::Float,
            _ => ValueType::Void,
        }
    }

    /// Max this type with another type, return more precise one.
    /// If types are not number, return void.
    pub fn max_with(&self, b: &Self) -> Self {
        if self.is_num() && b.is_num() {
            let a_lv = self.to_precision_level();
            let b_lv = b.to_precision_level();
            let max_lv = if a_lv > b_lv { a_lv } else { b_lv };
            ValueType::from_precision_level(max_lv)
        } else {
            ValueType::Void
        }
    }
}
