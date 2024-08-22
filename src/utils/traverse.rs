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

use std::{
    collections::{HashSet, VecDeque},
    hash::Hash,
};

pub trait Node: Eq + Hash + Clone {
    fn get_succ(&mut self) -> Vec<Self>;
}

/// Postorder iterator.
pub struct POIterator<T>
where
    T: Node,
{
    container: VecDeque<T>,
}

impl<T> Iterator for POIterator<T>
where
    T: Node,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.container.pop_front()
    }
}

impl<T> From<T> for POIterator<T>
where
    T: Node,
{
    fn from(bb: T) -> Self {
        // Run postorder traversal
        let mut container = Vec::new();
        let mut visited = HashSet::new();
        run_postorder(bb, &mut visited, &mut container);

        // Wrap in iterator
        Self {
            container: container.into(),
        }
    }
}

/// Reverse postorder iterator.
pub struct RPOIterator<T>
where
    T: Node,
{
    container: Vec<T>,
}

impl<T> Iterator for RPOIterator<T>
where
    T: Node,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.container.pop()
    }
}

impl<T> From<T> for RPOIterator<T>
where
    T: Node,
{
    fn from(bb: T) -> Self {
        // Run postorder traversal
        let mut container = Vec::new();
        let mut visited = HashSet::new();
        run_postorder(bb, &mut visited, &mut container);

        // Wrap in iterator
        Self { container }
    }
}

/// Run a complete post order traversal.
fn run_postorder<T>(mut bb: T, visited: &mut HashSet<T>, container: &mut Vec<T>)
where
    T: Node,
{
    if visited.contains(&bb) {
        return;
    }
    visited.insert(bb.clone());
    for succ in bb.get_succ() {
        run_postorder(succ.clone(), visited, container);
    }
    container.push(bb);
}
