// Copyright 2024 Duskphantom Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use std::collections::VecDeque;

use anyhow::Result;

use crate::{Expr, Type};

/// Reshape a possibly flattened constant array to nested.
///
/// # Panics
/// Please make sure `arr` is non-empty.
pub fn reshape_const_array(arr: &mut VecDeque<Expr>, ty: &Type) -> Result<Expr> {
    if arr.is_empty() {
        // Use default initializer for `{}`
        return ty.default_initializer();
    }
    if let Type::Array(element_ty, len) = ty {
        let size = len.to_i32()?;
        let mut new_arr: Vec<Expr> = vec![];
        for _ in 0..size {
            let Some(first_item) = arr.pop_front() else {
                // Later elements are missing, fill with default initializer
                new_arr.push(element_ty.default_initializer()?);
                continue;
            };
            if let Expr::Array(arr) = first_item {
                // First element is array, sub-array is nested
                new_arr.push(reshape_const_array(&mut VecDeque::from(arr), element_ty)?);
            } else {
                // First element is non-array, sub-array is flattened
                arr.push_front(first_item);
                new_arr.push(reshape_const_array(arr, element_ty)?);
            }
        }
        Ok(Expr::Array(new_arr))
    } else {
        Ok(arr.pop_front().unwrap())
    }
}

/// Reshape a possibly flattened array to nested.
///
/// # Panics
/// Please make sure `arr` is non-empty.
pub fn reshape_array(arr: &mut VecDeque<Expr>, ty: &Type) -> Result<Expr> {
    if let Type::Array(element_ty, len) = ty {
        let size = len.to_i32()?;
        let mut new_arr: Vec<Expr> = vec![];
        for _ in 0..size {
            let Some(first_item) = arr.pop_front() else {
                break;
            };
            if let Expr::Array(arr) = first_item {
                // First element is array, sub-array is nested
                new_arr.push(reshape_array(&mut VecDeque::from(arr), element_ty)?);
            } else {
                // First element is non-array, sub-array is flattened
                arr.push_front(first_item);
                new_arr.push(reshape_array(arr, element_ty)?);
            }
        }
        Ok(Expr::Array(new_arr))
    } else {
        Ok(arr.pop_front().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use crate::{transform::reshape_array::reshape_array, Expr, Type};

    #[test]
    fn test_reshape_flattened_array() {
        let arr = vec![Expr::Int(1), Expr::Int(2), Expr::Int(3), Expr::Int(4)];
        let mut vec_deque = VecDeque::from(arr);
        let ty = Type::Array(
            Type::Array(Type::Int.into(), Expr::Int(2).into()).into(),
            Expr::Int(2).into(),
        );
        let res = reshape_array(&mut vec_deque, &ty).unwrap();
        assert_eq!(
            res,
            Expr::Array(vec![
                Expr::Array(vec![Expr::Int(1), Expr::Int(2)]),
                Expr::Array(vec![Expr::Int(3), Expr::Int(4)]),
            ])
        );
    }

    #[test]
    fn test_reshape_nested_array() {
        let arr = vec![
            Expr::Array(vec![Expr::Int(1), Expr::Int(2)]),
            Expr::Array(vec![Expr::Int(3), Expr::Int(4)]),
        ];
        let mut vec_deque = VecDeque::from(arr);
        let ty = Type::Array(
            Type::Array(Type::Int.into(), Expr::Int(2).into()).into(),
            Expr::Int(2).into(),
        );
        let res = reshape_array(&mut vec_deque, &ty).unwrap();
        assert_eq!(
            res,
            Expr::Array(vec![
                Expr::Array(vec![Expr::Int(1), Expr::Int(2)]),
                Expr::Array(vec![Expr::Int(3), Expr::Int(4)]),
            ])
        );
    }

    #[test]
    fn test_reshape_mixed_array() {
        let arr = vec![
            Expr::Int(1),
            Expr::Int(2),
            Expr::Array(vec![Expr::Int(3), Expr::Int(4)]),
        ];
        let mut vec_deque = VecDeque::from(arr);
        let ty = Type::Array(
            Type::Array(Type::Int.into(), Expr::Int(2).into()).into(),
            Expr::Int(2).into(),
        );
        let res = reshape_array(&mut vec_deque, &ty).unwrap();
        assert_eq!(
            res,
            Expr::Array(vec![
                Expr::Array(vec![Expr::Int(1), Expr::Int(2)]),
                Expr::Array(vec![Expr::Int(3), Expr::Int(4)]),
            ])
        );
    }

    #[test]
    fn test_reshape_mixed_array_2() {
        let arr = vec![
            Expr::Array(vec![Expr::Int(1), Expr::Int(2)]),
            Expr::Int(3),
            Expr::Int(4),
        ];
        let mut vec_deque = VecDeque::from(arr);
        let ty = Type::Array(
            Type::Array(Type::Int.into(), Expr::Int(2).into()).into(),
            Expr::Int(2).into(),
        );
        let res = reshape_array(&mut vec_deque, &ty).unwrap();
        assert_eq!(
            res,
            Expr::Array(vec![
                Expr::Array(vec![Expr::Int(1), Expr::Int(2)]),
                Expr::Array(vec![Expr::Int(3), Expr::Int(4)]),
            ])
        );
    }

    #[test]
    fn test_reshape_fractured_array() {
        let arr = vec![Expr::Array(vec![Expr::Int(1)]), Expr::Int(3)];
        let mut vec_deque = VecDeque::from(arr);
        let ty = Type::Array(
            Type::Array(Type::Int.into(), Expr::Int(2).into()).into(),
            Expr::Int(2).into(),
        );
        let res = reshape_array(&mut vec_deque, &ty).unwrap();
        assert_eq!(
            res,
            Expr::Array(vec![
                Expr::Array(vec![Expr::Int(1),]),
                Expr::Array(vec![Expr::Int(3),]),
            ])
        );
    }
}
