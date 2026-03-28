use my_app_core::{ActionRegistry, AppError};
use tracing::info;

pub struct AppService {
    registry: ActionRegistry,
}

impl AppService {
    pub fn new(registry: ActionRegistry) -> Self {
        Self { registry }
    }

    pub fn execute(&self, action: &str, payload: Vec<u8>) -> Result<Vec<u8>, AppError> {
        info!(action = action, "executing action");
        self.registry.execute(action, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use my_app_core::actions::echo::EchoAction;

    fn build_service() -> AppService {
        let mut registry = ActionRegistry::new();
        registry.register(Box::new(EchoAction));
        AppService::new(registry)
    }

    #[test]
    fn test_app_service_echo() {
        let svc = build_service();
        let input = serde_json::json!({"message": "world"}).to_string().into_bytes();
        let output = svc.execute("echo", input).unwrap();
        let val: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(val["message"], "world");
    }

    #[test]
    fn test_app_service_action_not_found() {
        let svc = build_service();
        let result = svc.execute("missing", vec![]);
        assert!(matches!(result, Err(AppError::ActionNotFound(_))));
    }
}
