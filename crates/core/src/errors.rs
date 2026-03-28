#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("invalid payload")]
    InvalidPayload,
    #[error("execution failed: {0}")]
    Execution(String),
}
