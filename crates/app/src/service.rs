use my_app_core::AppError;
use my_app_core::actions::{info::InfoAction, deploy_agent::DeployAgentAction};
use tracing::info;

pub struct AppService {
    info_action: InfoAction,
    deploy_agent_action: DeployAgentAction,
}

impl AppService {
    pub fn new(info_action: InfoAction, deploy_agent_action: DeployAgentAction) -> Self {
        Self { info_action, deploy_agent_action }
    }

    pub fn get_info(&self) -> Result<Vec<u8>, AppError> {
        info!("executing info action");
        self.info_action.get_info()
    }

    pub fn deploy_agent(&self, payload: Vec<u8>) -> Result<Vec<u8>, AppError> {
        info!("executing deploy-agent action");
        self.deploy_agent_action.deploy(payload)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use my_app_core::actions::launched_apps::LaunchedApps;
    use my_app_core::models::ApplicationAccess;

    fn build_service() -> AppService {
        let launched_apps = Arc::new(LaunchedApps::new(vec![ApplicationAccess {
            application: "platform-manager".to_string(),
            url: "http://localhost:50051".to_string(),
        }]));
        AppService::new(
            InfoAction::new(Arc::clone(&launched_apps)),
            DeployAgentAction::new(launched_apps),
        )
    }

    #[test]
    fn test_app_service_get_info() {
        let svc = build_service();
        let output = svc.get_info().unwrap();
        let val: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(val["application"], "platform-manager");
    }

    #[test]
    fn test_app_service_deploy_agent() {
        let svc = build_service();
        let input = serde_json::json!({"application": "orders-api", "url": "https://orders.example.com"})
            .to_string()
            .into_bytes();
        let output = svc.deploy_agent(input).unwrap();
        let val: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(val["application"], "orders-api");
    }
}
