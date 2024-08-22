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

// 实现一个fprintln宏用于写入文件
#[macro_export]
macro_rules! fprint {
    ($file:expr, $($arg:tt)*) => {
        #[cfg(feature = "log_enabled")]
        {
            let s=format!($($arg)*);
            $crate::utils::log::must_write($file, &s,false);
        }
    };
    ($file:expr;$mode:literal;$($arg:tt)*) => {
        #[cfg(feature = "log_enabled")]
        {
            let s=format!($($arg)*);
            let append=$mode=='a';
            $crate::utils::log::must_write($file, &s,append);
        }
    };
}
#[macro_export]
macro_rules! fprintln {
    ($file:expr, $($arg:tt)*) => {
        #[cfg(feature = "log_enabled")]
        {
            let mut s=format!($($arg)*);
            s.push('\n');
            $crate::utils::log::must_write($file, &s,false);
        }
    };
    ($file:expr;$mode:literal;$($arg:tt)*) => {
        #[cfg(feature = "log_enabled")]
        {
            let mut s=format!($($arg)*);
            s.push('\n');
            let append=$mode=='a';
            $crate::utils::log::must_write($file, &s,append);
        }
    };
}
// 控制台打印（条件编译）
#[macro_export]
macro_rules! cprintln {
    ($($arg:tt)*) => {
        #[cfg(feature = "log_enabled")]
        {
            println!($($arg)*);
        }
    };
}

#[allow(unused)]
pub fn must_write(path: &str, content: &str, append: bool) {
    use std::fs::OpenOptions;
    use std::io::Write;
    // if dir not exists, create it
    let path = std::path::Path::new(path);
    if let Some(dir) = path.parent() {
        if !dir.exists() {
            std::fs::create_dir_all(dir).unwrap();
        }
    }
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .open(path)
        .unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file.flush().unwrap();
}
