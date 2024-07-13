use crate::middle::ir::ValueType;

/// Convenient methods for value type
impl ValueType {
    /// If a value type can be converted to a number, returns true
    pub fn is_num(&self) -> bool {
        matches!(self, ValueType::Bool | ValueType::Int | ValueType::Float)
    }

    /// Convert a numeric value type to its precision level
    /// Higher is more precise
    pub fn to_precision_level(&self) -> i32 {
        match self {
            // All boolean should be converted to int when applying `+` and etc.
            ValueType::Bool => 1,
            ValueType::Int => 1,
            ValueType::Float => 2,
            _ => 0,
        }
    }

    /// Convert a precision level to a value type
    pub fn from_precision_level(level: i32) -> Self {
        match level {
            1 => ValueType::Int,
            2 => ValueType::Float,
            _ => ValueType::Void,
        }
    }

    /// Max this type with another type
    /// Return more precise one
    /// If types are not number, return void
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
