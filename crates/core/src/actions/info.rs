use std::time::{SystemTime, UNIX_EPOCH};

use super::Action;
use crate::errors::AppError;
use crate::models::{InfoEndpoint, InfoResponse};

pub struct InfoAction;

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
            task_id: format!("task-{millis}"),
        };

        serde_json::to_vec(&resp).map_err(|e| AppError::Execution(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_success() {
        let action = InfoAction;
        let output = action.execute(vec![]).unwrap();
        let resp: InfoResponse = serde_json::from_slice(&output).unwrap();
        assert_eq!(resp.application, "platform-manager");
        assert_eq!(resp.endpoints.len(), 2);
        assert!(resp.task_id.starts_with("task-"));
    }

    #[test]
    fn test_info_rejects_non_empty_payload() {
        let action = InfoAction;
        let result = action.execute(br#"{"ignored":true}"#.to_vec());
        assert!(matches!(result, Err(AppError::InvalidPayload)));
    }
}