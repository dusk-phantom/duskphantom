use anyhow::Result;

use crate::middle::ir::{Constant, ValueType};
use std::collections::VecDeque;

/// Collapse a possibly flattened constant array to nested
///
/// # Panics
/// Please make sure `arr` is non-empty.
pub fn collapse_array(arr: &mut VecDeque<Constant>, ty: &ValueType) -> Result<Constant> {
    if let ValueType::Array(element_ty, len) = ty {
        let mut new_arr: Vec<Constant> = vec![];
        for _ in 0..*len {
            let Some(first_item) = arr.pop_front() else {
                // TODO use zero initializer
                new_arr.push(collapse_array(arr, element_ty)?);
                continue;
            };
            if let Constant::Array(arr) = first_item {
                // First element is array, sub-array is nested
                new_arr.push(collapse_array(&mut VecDeque::from(arr), element_ty)?);
            } else {
                // First element is non-array, sub-array is flattened
                arr.push_front(first_item);
                new_arr.push(collapse_array(arr, element_ty)?);
            }
        }
        Ok(Constant::Array(new_arr))
    } else if let Some(val) = arr.pop_front() {
        Ok(val)
    } else {
        // TODO use zero initializer
        ty.default_initializer()
    }
}
