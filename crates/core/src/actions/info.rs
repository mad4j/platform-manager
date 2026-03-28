use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;

use super::Action;
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
}

impl Action for InfoAction {
    fn name(&self) -> &'static str {
        "info"
    }

    fn execute(&self, input: Vec<u8>) -> Result<Vec<u8>, AppError> {
        if !input.is_empty() {
            return Err(AppError::InvalidPayload);
        }

        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AppError::Execution(e.to_string()))?
            .as_millis();

        let resp = InfoResponse {
            application: "platform-manager".to_string(),
            endpoints: vec![
                InfoEndpoint {
                    name: "grpc_info_rpc".to_string(),
                    value: "/action.InfoService/Info (InfoRequest -> InfoResponse)".to_string(),
                },
                InfoEndpoint {
                    name: "grpc_execute_rpc".to_string(),
                    value: "/action.ActionService/Execute (generic action endpoint)".to_string(),
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
        let output = action.execute(vec![]).unwrap();
        let resp: InfoResponse = serde_json::from_slice(&output).unwrap();
        assert_eq!(resp.application, "platform-manager");
        assert_eq!(resp.endpoints.len(), 2);
        assert_eq!(
            resp.launched_applications,
            vec![ApplicationAccess {
                application: "platform-manager".to_string(),
                url: "http://localhost:50051".to_string(),
            }]
        );
        assert!(resp.task_id.starts_with("task-"));
    }

    #[test]
    fn test_info_rejects_non_empty_payload() {
        let launched_apps = Arc::new(LaunchedApps::new(vec![]));
        let action = InfoAction::new(launched_apps);
        let result = action.execute(br#"{"ignored":true}"#.to_vec());
        assert!(matches!(result, Err(AppError::InvalidPayload)));
    }
}