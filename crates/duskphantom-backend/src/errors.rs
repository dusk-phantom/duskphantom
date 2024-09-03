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

use thiserror::Error;

// 后端错误
#[derive(Debug, Error)]
pub enum BackendError {
    // 生成错误
    #[error("gen error")]
    GenError,
    // 优化错误
    #[error("optimize error")]
    OptimizeError,
    #[error("gen from llvm error: {0}")]
    GenFromLlvmError(String),
    #[error("internal consistency error: {0}")]
    InternalConsistencyError(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
