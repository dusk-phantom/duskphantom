use super::*;

#[derive(Clone, Copy)]
pub enum Constant {
    Int(i32),
    Float(f32),
    Bool(bool),
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::Int(i) => write!(f, "{}", i),
            Constant::Float(fl) => write!(f, "{}", fl),
            Constant::Bool(b) => write!(f, "{}", b),
        }
    }
}

impl Constant {
    pub fn get_type(&self) -> ValueType {
        match self {
            Constant::Int(_) => ValueType::Int,
            Constant::Float(_) => ValueType::Float,
            Constant::Bool(_) => ValueType::Bool,
        }
    }
}
