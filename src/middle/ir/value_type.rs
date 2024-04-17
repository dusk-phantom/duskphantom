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
}
