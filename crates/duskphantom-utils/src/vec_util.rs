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

use anyhow::Result;

pub fn insert_vec<T: Clone>(vec: &mut Vec<T>, mut to_insert: Vec<(usize, T)>) -> Result<()> {
    if to_insert.is_empty() {
        return Ok(());
    }
    to_insert.sort_by(|a, b| a.0.cmp(&b.0));
    assert!(to_insert[0].0 < vec.len());
    let mut new_vec = Vec::with_capacity(vec.len() + to_insert.len());
    for (i, item) in vec.iter().enumerate() {
        while let Some((index, _)) = to_insert.first() {
            if index == &i {
                let (_, to_insert_item) = to_insert.remove(0);
                new_vec.push(to_insert_item);
            } else {
                break;
            }
        }
        new_vec.push(item.clone());
    }
    *vec = new_vec;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_vec() {
        let mut vec = vec![1, 2, 3, 4, 5];
        insert_vec(&mut vec, vec![(0, 0), (2, 2), (4, 4)].into_iter().collect()).unwrap();
        assert_eq!(vec, vec![0, 1, 2, 2, 3, 4, 4, 5]);
    }
}
