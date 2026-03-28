pub mod echo;
pub mod deploy_agent;
pub mod info;
pub mod launched_apps;

use crate::errors::AppError;

pub trait Action: Send + Sync {
    fn name(&self) -> &'static str;
    fn execute(&self, input: Vec<u8>) -> Result<Vec<u8>, AppError>;
}
