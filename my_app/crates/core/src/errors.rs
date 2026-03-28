#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("action not found: {0}")]
    ActionNotFound(String),
    #[error("invalid payload")]
    InvalidPayload,
    #[error("execution failed: {0}")]
    Execution(String),
}
