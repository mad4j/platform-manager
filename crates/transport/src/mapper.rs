use crate::manager::{Endpoint, InfoResponse, LaunchedApplication};
use platform_manager_core::AppError;
use platform_manager_core::models::InfoResponse as CoreInfoResponse;

pub fn to_info_proto(res: Result<Vec<u8>, AppError>) -> InfoResponse {
    match res {
        Ok(payload) => {
            let parsed: Result<CoreInfoResponse, _> = serde_json::from_slice(&payload);
            match parsed {
                Ok(info) => InfoResponse {
                    application: info.application,
                    endpoints: info
                        .endpoints
                        .into_iter()
                        .map(|e| Endpoint {
                            name: e.name,
                            value: e.value,
                        })
                        .collect(),
                    task_id: info.task_id,
                    error: String::new(),
                    launched_applications: info
                        .launched_applications
                        .into_iter()
                        .map(|app| LaunchedApplication {
                            application: app.application,
                            url: app.url,
                        })
                        .collect(),
                },
                Err(e) => InfoResponse {
                    application: String::new(),
                    endpoints: vec![],
                    task_id: String::new(),
                    error: format!("invalid info payload: {e}"),
                    launched_applications: vec![],
                },
            }
        }
        Err(e) => InfoResponse {
            application: String::new(),
            endpoints: vec![],
            task_id: String::new(),
            error: e.to_string(),
            launched_applications: vec![],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use platform_manager_core::models::{
        ApplicationAccess as CoreApplicationAccess,
        InfoEndpoint,
        InfoResponse as CoreInfoResponse,
    };

    #[test]
    fn test_to_info_proto_success() {
        let payload = serde_json::to_vec(&CoreInfoResponse {
            application: "platform-manager".to_string(),
            endpoints: vec![InfoEndpoint {
                name: "grpc_info_rpc".to_string(),
                value: "/manager.InfoService/Info (InfoRequest -> InfoResponse)".to_string(),
            }],
            launched_applications: vec![CoreApplicationAccess {
                application: "platform-manager".to_string(),
                url: "http://localhost:50051".to_string(),
            }],
            task_id: "task-1".to_string(),
        })
        .unwrap();

        let resp = to_info_proto(Ok(payload));
        assert!(resp.error.is_empty());
        assert_eq!(resp.application, "platform-manager");
        assert_eq!(resp.endpoints.len(), 1);
        assert_eq!(resp.task_id, "task-1");
        assert_eq!(resp.launched_applications.len(), 1);
        assert_eq!(resp.launched_applications[0].application, "platform-manager");
        assert_eq!(resp.launched_applications[0].url, "http://localhost:50051");
    }

    #[test]
    fn test_to_info_proto_error() {
        let resp = to_info_proto(Err(AppError::InvalidPayload));
        assert!(resp.application.is_empty());
        assert!(resp.endpoints.is_empty());
        assert!(resp.task_id.is_empty());
        assert!(resp.launched_applications.is_empty());
        assert!(!resp.error.is_empty());
    }

    #[test]
    fn test_to_info_proto_invalid_payload() {
        let resp = to_info_proto(Ok(b"not json".to_vec()));
        assert!(resp.application.is_empty());
        assert!(resp.endpoints.is_empty());
        assert!(resp.task_id.is_empty());
        assert!(resp.launched_applications.is_empty());
        assert!(resp.error.starts_with("invalid info payload:"));
    }
}

