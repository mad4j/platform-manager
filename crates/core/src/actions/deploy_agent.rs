use std::sync::Arc;

use super::launched_apps::LaunchedApps;
use crate::errors::AppError;
use crate::models::{DeployAgentRequest, DeployAgentResponse};

pub struct DeployAgentAction {
    launched_apps: Arc<LaunchedApps>,
}

impl DeployAgentAction {
    pub fn new(launched_apps: Arc<LaunchedApps>) -> Self {
        Self { launched_apps }
    }

    pub fn deploy(&self, input: Vec<u8>) -> Result<Vec<u8>, AppError> {
        let req: DeployAgentRequest =
            serde_json::from_slice(&input).map_err(|_| AppError::InvalidPayload)?;

        let app_name = req.application.trim();
        if app_name.is_empty() {
            return Err(AppError::InvalidPayload);
        }

        let app_url = match req.url {
            Some(url) => {
                let url = url.trim();
                if url.is_empty() {
                    return Err(AppError::InvalidPayload);
                }
                url.to_string()
            }
            None => format!("http://localhost/{app_name}"),
        };

        self.launched_apps
            .record(app_name.to_string(), app_url.clone());

        let resp = DeployAgentResponse {
            application: app_name.to_string(),
            url: app_url,
            status: "deployed".to_string(),
            message: format!("application '{app_name}' deployed by deploy-agent"),
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
    fn test_deploy_agent_success() {
        let launched_apps = Arc::new(LaunchedApps::new(vec![ApplicationAccess {
            application: "platform-manager".to_string(),
            url: "http://localhost:50051".to_string(),
        }]));
        let action = DeployAgentAction::new(Arc::clone(&launched_apps));

        let input = serde_json::to_vec(&DeployAgentRequest {
            application: "billing-api".to_string(),
            url: Some("https://billing.example.com".to_string()),
        })
        .unwrap();

        let output = action.deploy(input).unwrap();
        let resp: DeployAgentResponse = serde_json::from_slice(&output).unwrap();

        assert_eq!(resp.application, "billing-api");
        assert_eq!(resp.url, "https://billing.example.com");
        assert_eq!(resp.status, "deployed");
        assert!(resp.message.contains("billing-api"));
        assert!(launched_apps.list().contains(&ApplicationAccess {
            application: "billing-api".to_string(),
            url: "https://billing.example.com".to_string(),
        }));
    }

    #[test]
    fn test_deploy_agent_rejects_invalid_payload() {
        let launched_apps = Arc::new(LaunchedApps::new(vec![]));
        let action = DeployAgentAction::new(launched_apps);

        let result = action.deploy(b"not json".to_vec());
        assert!(matches!(result, Err(AppError::InvalidPayload)));
    }

    #[test]
    fn test_deploy_agent_rejects_empty_application() {
        let launched_apps = Arc::new(LaunchedApps::new(vec![]));
        let action = DeployAgentAction::new(launched_apps);

        let input = serde_json::to_vec(&DeployAgentRequest {
            application: "   ".to_string(),
            url: None,
        })
        .unwrap();

        let result = action.deploy(input);
        assert!(matches!(result, Err(AppError::InvalidPayload)));
    }

    #[test]
    fn test_deploy_agent_generates_default_url() {
        let launched_apps = Arc::new(LaunchedApps::new(vec![]));
        let action = DeployAgentAction::new(launched_apps);

        let input = serde_json::to_vec(&DeployAgentRequest {
            application: "orders-api".to_string(),
            url: None,
        })
        .unwrap();

        let output = action.deploy(input).unwrap();
        let resp: DeployAgentResponse = serde_json::from_slice(&output).unwrap();
        assert_eq!(resp.url, "http://localhost/orders-api");
    }
}
