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

use std::ops::{Deref, DerefMut};

use super::*;
use crate::define_graph_iterator;

pub type FunPtr = ObjPtr<Function>;

pub struct Function {
    pub mem_pool: ObjPtr<IRBuilder>,

    pub name: String,

    /// Entry of function, if it is a function that is not defined in this module, it will be None.
    /// Such as library function.
    pub entry: Option<BBPtr>,

    /// Exit of function, if it is a function that is not defined in this module, it will be None.
    /// Such as library function.
    pub exit: Option<BBPtr>,

    pub return_type: ValueType,

    /// BasicBlock of function parameters
    pub params: Vec<ParaPtr>,
}

impl Function {
    /// Return true if it is a function that is not defined in this module.
    pub fn is_lib(&self) -> bool {
        self.entry.is_none()
    }

    /// Return true if it is main function.
    pub fn is_main(&self) -> bool {
        self.name == "main"
    }

    /// Create a depth-first iterator to traverse the graph structure of basicblocks.
    /// Traverse in the direction of data flow with the function entry as the starting point.
    /// Do not change the graph structure during traversal, which may cause unknown errors
    pub fn dfs_iter(&self) -> DFSIterator {
        DFSIterator::from(self.entry.unwrap())
    }

    /// Create a breadth-first iterator to traverse the graph structure of basicblocks.
    /// Traverse in the direction of data flow with the function entry as the starting point.
    /// Do not change the graph structure during traversal, which may cause unknown errors
    pub fn bfs_iter(&self) -> BFSIterator {
        BFSIterator::from(self.entry.unwrap())
    }

    /// Create a depth-first iterator to traverse the graph structure of basicblocks.
    /// Traverse in the reverse direction of data flow with the function exit as the starting point.
    /// Do not change the graph structure during traversal, which may cause unknown errors
    pub fn dfs_iter_rev(&self) -> DFSIteratorRev {
        DFSIteratorRev::from(self.exit.unwrap())
    }

    /// Create a breadth-first iterator to traverse the graph structure of basicblocks.
    /// Traverse in the reverse direction of data flow with the function exit as the starting point.
    /// Do not change the graph structure during traversal, which may cause unknown errors
    pub fn bfs_iter_rev(&self) -> BFSIteratorRev {
        BFSIteratorRev::from(self.exit.unwrap())
    }

    /// Create a postorder iterator to traverse the graph structure of basicblocks.
    pub fn po_iter(&self) -> POIterator {
        POIterator::from(self.entry.unwrap())
    }

    /// Create a reverse postorder iterator to traverse the graph structure of basicblocks.
    pub fn rpo_iter(&self) -> RPOIterator {
        RPOIterator::from(self.entry.unwrap())
    }

    pub fn gen_llvm_ir(&self) -> String {
        let header = if self.is_lib() { "declare" } else { "define" };
        let mut ir = format!("{} {} @{}(", header, self.return_type, self.name);
        if !self.params.is_empty() {
            for param in self.params.iter() {
                ir += &format!("{}, ", param.as_ref());
            }
            let _ = ir.split_off(ir.len() - 2);
        }
        ir += ")";

        // If it is a library function, there is no need to generate the body
        if self.is_lib() {
            ir += "\n";
            return ir;
        }

        // Otherwise, generate the body of the function
        ir += " {\n";
        self.bfs_iter().for_each(|bb| {
            ir += &bb.gen_llvm_ir();
        });
        ir + "\n}\n"
    }
}

define_graph_iterator!(BFSIterator, VecDeque<BBPtr>, pop_front, get_succ_bb);
define_graph_iterator!(BFSIteratorRev, VecDeque<BBPtr>, pop_front, get_pred_bb);
define_graph_iterator!(DFSIterator, Vec<BBPtr>, pop, get_succ_bb);
define_graph_iterator!(DFSIteratorRev, Vec<BBPtr>, pop, get_pred_bb);

/// Postorder iterator.
pub struct POIterator {
    container: VecDeque<BBPtr>,
}

impl Iterator for POIterator {
    type Item = BBPtr;
    fn next(&mut self) -> Option<Self::Item> {
        self.container.pop_front()
    }
}

impl From<BBPtr> for POIterator {
    fn from(bb: BBPtr) -> Self {
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
pub struct RPOIterator {
    container: Vec<BBPtr>,
}

impl Iterator for RPOIterator {
    type Item = BBPtr;
    fn next(&mut self) -> Option<Self::Item> {
        self.container.pop()
    }
}

impl From<BBPtr> for RPOIterator {
    fn from(bb: BBPtr) -> Self {
        // Run postorder traversal
        let mut container = Vec::new();
        let mut visited = HashSet::new();
        run_postorder(bb, &mut visited, &mut container);

        // Wrap in iterator
        Self { container }
    }
}

/// Run a complete post order traversal.
fn run_postorder(bb: BBPtr, visited: &mut HashSet<BBPtr>, container: &mut Vec<BBPtr>) {
    if visited.contains(&bb) {
        return;
    }
    visited.insert(bb);
    for succ in bb.get_succ_bb() {
        run_postorder(*succ, visited, container);
    }
    container.push(bb);
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ParaPtr(ObjPtr<Parameter>);
impl Display for ParaPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{}", self.0.name)
    }
}
impl Deref for ParaPtr {
    type Target = Parameter;
    fn deref(&self) -> &Parameter {
        self.0.as_ref()
    }
}
impl DerefMut for ParaPtr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut()
    }
}
impl AsRef<Parameter> for ParaPtr {
    fn as_ref(&self) -> &Parameter {
        self.0.as_ref()
    }
}
impl From<ObjPtr<Parameter>> for ParaPtr {
    fn from(ptr: ObjPtr<Parameter>) -> Self {
        Self(ptr)
    }
}

#[derive(Clone)]
pub struct Parameter {
    pub name: String,
    pub value_type: ValueType,
    user: Vec<InstPtr>,
}

impl Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} %{}", self.value_type, self.name)
    }
}

impl Parameter {
    pub fn new(name: String, value_type: ValueType) -> Self {
        Self {
            name,
            value_type,
            user: Vec::new(),
        }
    }

    pub fn get_user(&self) -> &[InstPtr] {
        &self.user
    }
    pub fn get_user_mut(&mut self) -> &mut Vec<InstPtr> {
        &mut self.user
    }
    /// # Safety
    /// FIXME: explain why it is unsafe,and describe the safety requirements
    pub unsafe fn add_user(&mut self, inst: InstPtr) {
        self.user.push(inst);
    }
    /// # Safety
    /// FIXME: explain why it is unsafe,and describe the safety requirements
    pub unsafe fn remove_user(&mut self, inst: InstPtr) {
        self.user
            .iter()
            .position(|x| *x == inst)
            .map(|i| self.user.swap_remove(i));
    }
}
