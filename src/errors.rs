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

// 前端错误
#[derive(Debug, Error)]
pub enum FrontendError {
    // 解析错误
    #[error("parse error")]
    ParseError(String),
    // 优化错误
    #[error("optimize error")]
    OptimizeError,
}

// 中端错误
#[derive(Debug, Error)]
pub enum MiddleError {
    // 生成错误
    #[error("gen error")]
    GenError,
    // 优化错误
    #[error("optimize error")]
    OptimizeError,
    // Custom error
    #[error("custom error")]
    CustomError(String),
}

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
            FrontendError::OptimizeError => {
                eprintln!("msg: optimize error");
            }
        },
        _ => (),
    }
    eprintln!("msg: compile failed");
    eprintln!("err: {}", err);
    std::process::exit(1);
}
