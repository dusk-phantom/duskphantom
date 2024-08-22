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

pub fn diff(a: &str, b: &str) -> String {
    diff::lines(a, b)
        .iter()
        .map(|line| match line {
            diff::Result::Left(s) => format!("[-] {}", s),
            diff::Result::Both(s, _) => s.to_string(),
            diff::Result::Right(s) => format!("[+] {}", s),
        })
        .collect::<Vec<String>>()
        .join("\n")
}
