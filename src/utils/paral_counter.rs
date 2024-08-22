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

//高并发id分配器,用于分配虚拟寄存器使用的id
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
// 能够高并发地管理一定 数据范围内的id分配
#[derive(Debug, Clone)]
pub struct ParalCounter {
    end: usize,
    counter: Arc<AtomicUsize>,
}

impl ParalCounter {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            end,
            counter: Arc::new(AtomicUsize::new(start)),
        }
    }
    pub fn get_id(&self) -> Option<usize> {
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        if id < self.end {
            Some(id)
        } else {
            None
        }
    }
}
