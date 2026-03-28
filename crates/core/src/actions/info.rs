use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;

use super::launched_apps::LaunchedApps;
use crate::errors::AppError;
use crate::models::{InfoEndpoint, InfoResponse};

pub struct InfoAction {
    launched_apps: Arc<LaunchedApps>,
}

impl InfoAction {
    pub fn new(launched_apps: Arc<LaunchedApps>) -> Self {
        Self { launched_apps }
    }

    pub fn get_info(&self) -> Result<Vec<u8>, AppError> {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AppError::Execution(e.to_string()))?
            .as_millis();

        let resp = InfoResponse {
            application: "platform-manager".to_string(),
            endpoints: vec![
                InfoEndpoint {
                    name: "grpc_info_rpc".to_string(),
                    value: "/manager.InfoService/Info (InfoRequest -> InfoResponse)".to_string(),
                },
            ],
            launched_applications: self.launched_apps.list(),
            task_id: format!("task-{millis}"),
        };

        serde_json::to_vec(&resp).map_err(|e| AppError::Execution(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::models::ApplicationAccess;

    #[test]
    fn test_info_success() {
        let launched_apps = Arc::new(LaunchedApps::new(vec![ApplicationAccess {
            application: "platform-manager".to_string(),
            url: "http://localhost:50051".to_string(),
        }]));
        let action = InfoAction::new(launched_apps);
        let output = action.get_info().unwrap();
        let resp: InfoResponse = serde_json::from_slice(&output).unwrap();
        assert_eq!(resp.application, "platform-manager");
        assert_eq!(resp.endpoints.len(), 1);
        assert_eq!(
            resp.launched_applications,
            vec![ApplicationAccess {
                application: "platform-manager".to_string(),
                url: "http://localhost:50051".to_string(),
            }]
        );
        assert!(resp.task_id.starts_with("task-"));
    }
}