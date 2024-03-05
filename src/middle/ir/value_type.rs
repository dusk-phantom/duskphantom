/// Represent the type of a value.
#[derive(Clone, PartialEq, Eq)]
pub enum ValueType {
    Void,
    Int,
    Float,
    Array(Box<ValueType>, usize),
    Pointer(Box<ValueType>),
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Void => write!(f, "void"),
            ValueType::Int => write!(f, "i32"),
            ValueType::Float => write!(f, "float"),
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
