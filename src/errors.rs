use thiserror::Error;

// 全局error处理表
#[derive(Debug, Error)]
pub enum CompilerError {
    // IO错误
    #[error("io error, cause: {0}")]
    IOError(#[from] std::io::Error),
    // 前端错误
    #[error("frontend error, cause: {0}")]
    FrontendError(#[from] FrontEndError),
    // 中端错误
    #[error("middle error, cause: {0}")]
    MiddleError(#[from] MiddelError),
    // 后端错误
    #[error("backend error, cause: {0}")]
    BackendError(#[from] BackendError),
}

// 前端错误
#[derive(Debug, Error)]
pub enum FrontEndError {
    // 解析错误
    #[error("parse error")]
    ParseError,
    // 优化错误
    #[error("optimize error")]
    OptimizeError,
}

// 中端错误
#[derive(Debug, Error)]
pub enum MiddelError {
    // 生成错误
    #[error("gen error")]
    GenError,
    // 优化错误
    #[error("optimize error")]
    OptimizeError,
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
}
