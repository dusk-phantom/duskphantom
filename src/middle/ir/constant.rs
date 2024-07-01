use super::*;

#[derive(Clone, Debug)]
pub enum Constant {
    Int(i32),
    Float(f32),
    Bool(bool),
    Array(Vec<Constant>),
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::Int(i) => write!(f, "{}", i),
            Constant::Float(fl) => write!(f, "{}", fl),
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
        }
    }
}

impl Constant {
    pub fn get_type(&self) -> ValueType {
        match self {
            Constant::Int(_) => ValueType::Int,
            Constant::Float(_) => ValueType::Float,
            Constant::Bool(_) => ValueType::Bool,
            Constant::Array(arr) => {
                let sub_type = arr.first().unwrap().get_type();
                ValueType::Array(Box::new(sub_type), arr.len())
            }
        }
    }
}

impl From<i32> for Constant {
    fn from(i: i32) -> Self {
        Self::Int(i)
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
