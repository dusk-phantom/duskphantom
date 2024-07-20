use anyhow::{anyhow, Context, Result};

use crate::context;

use super::Constant;

/// Represent the type of a value.
#[derive(Clone, PartialEq, Eq)]
pub enum ValueType {
    Void,
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

    pub fn is_pointer(&self) -> bool {
        matches!(self, ValueType::Pointer(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, ValueType::Array(_, _))
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

    /// Get default initializer of this type.
    pub fn default_initializer(&self) -> Result<Constant> {
        match self {
            ValueType::Void => {
                Err(anyhow!("Cannot convert void type to constant")).with_context(|| context!())
            }
            ValueType::Int => Ok(Constant::Int(0)),
            ValueType::Float => Ok(Constant::Float(0.0)),
            ValueType::Bool => Ok(Constant::Bool(false)),
            ValueType::Pointer(_) => {
                Err(anyhow!("Cannot convert pointer type to constant")).with_context(|| context!())
            }
            ValueType::Array(ty, num) => {
                let inner_const = ty.default_initializer()?;
                Ok(Constant::Array(vec![inner_const; *num]))
            }
        }
    }
}
