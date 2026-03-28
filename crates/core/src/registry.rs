use std::collections::HashMap;
use crate::actions::Action;
use crate::errors::AppError;

pub struct ActionRegistry {
    actions: HashMap<String, Box<dyn Action>>,
}

impl ActionRegistry {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    pub fn register(&mut self, action: Box<dyn Action>) {
        self.actions.insert(action.name().to_string(), action);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Action> {
        self.actions.get(name).map(|a| a.as_ref())
    }

    pub fn execute(&self, name: &str, input: Vec<u8>) -> Result<Vec<u8>, AppError> {
        let action = self.get(name).ok_or_else(|| AppError::ActionNotFound(name.to_string()))?;
        action.execute(input)
    }
}

impl Default for ActionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::echo::EchoAction;

    #[test]
    fn test_registry_action_not_found() {
        let registry = ActionRegistry::new();
        let result = registry.execute("nonexistent", vec![]);
        assert!(matches!(result, Err(AppError::ActionNotFound(_))));
    }

    #[test]
    fn test_registry_echo() {
        let mut registry = ActionRegistry::new();
        registry.register(Box::new(EchoAction));
        let input = serde_json::json!({"message": "test"}).to_string().into_bytes();
        let output = registry.execute("echo", input).unwrap();
        let val: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(val["message"], "test");
    }
}
