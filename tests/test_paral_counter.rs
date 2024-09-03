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

extern crate duskphantom as compiler;
#[cfg(test)]
pub mod tests {
    use duskphantom_utils::paral_counter::ParalCounter;
    use std::collections::HashSet;
    use std::sync::Arc;
    #[test]
    fn test_paral_counter() {
        let start = 0;
        let end = 1_000_000;
        let counter = ParalCounter::new(0, end);
        // 开启多个线程,每个线程分配id,收集最终id
        let mut ids: HashSet<usize> = HashSet::with_capacity(end - start + 1);
        let mut handles = Vec::new();
        let counter = Arc::new(counter);
        for _i in 0..8 {
            let counter = Arc::clone(&counter);
            let handle = std::thread::spawn(move || {
                let mut ids = HashSet::new();
                while let Some(id) = counter.get_id() {
                    ids.insert(id);
                }
                ids
            });
            handles.push(handle);
        }
        for handle in handles {
            let par_ids = handle.join();
            if let Ok(par_ids) = par_ids {
                ids.extend(par_ids.iter().clone())
            } else if let Err(e) = par_ids {
                panic!("thread panic: {:?}", e);
            }
        }
        for i in start..end {
            assert!(ids.contains(&i));
        }
    }
}
