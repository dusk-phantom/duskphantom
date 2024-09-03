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

pub use duskphantom_backend::BackendError;
pub use duskphantom_frontend::errors::FrontendError;
pub use duskphantom_middle::errors::MiddleError;
use thiserror::Error;

// 全局error处理表
#[derive(Debug, Error)]
pub enum CompilerError {
    // IO错误
    #[error("io error, cause: {0}")]
    IOError(#[from] std::io::Error),
    // 前端错误
    #[error("frontend error, cause: {0}")]
    FrontendError(#[from] FrontendError),
    // 中端错误
    #[error("middle error, cause: {0}")]
    MiddleError(#[from] MiddleError),
    // 后端错误
    #[error("backend error, cause: {0}")]
    BackendError(#[from] BackendError),
    // from anyhow
    #[error("{0:?}")]
    Other(#[from] anyhow::Error),
}

/// 全局 错误处理函数
pub fn handle_error(err: &CompilerError) {
    match err {
        CompilerError::IOError(err) => {
            eprintln!("msg: io error");
            eprintln!("err: {}", err);
        }
        CompilerError::FrontendError(err) => match err {
            FrontendError::ParseError(msg) => {
                eprintln!("msg: parse error: {}", msg);
            }
            FrontendError::OptimizeError(e) => {
                eprintln!("msg: optimize error: {e}");
            }
        },
        _ => (),
    }
    eprintln!("msg: compile failed");
    eprintln!("err: {}", err);
    std::process::exit(1);
}
