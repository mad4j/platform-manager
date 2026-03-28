use super::Action;
use crate::errors::AppError;
use crate::models::{EchoRequest, EchoResponse};

pub struct EchoAction;

impl Action for EchoAction {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn execute(&self, input: Vec<u8>) -> Result<Vec<u8>, AppError> {
        let req: EchoRequest = serde_json::from_slice(&input)
            .map_err(|_| AppError::InvalidPayload)?;
        let resp = EchoResponse { message: req.message };
        serde_json::to_vec(&resp).map_err(|e| AppError::Execution(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo_success() {
        let action = EchoAction;
        let input = serde_json::to_vec(&EchoRequest { message: "hello".to_string() }).unwrap();
        let output = action.execute(input).unwrap();
        let resp: EchoResponse = serde_json::from_slice(&output).unwrap();
        assert_eq!(resp.message, "hello");
    }

    #[test]
    fn test_echo_invalid_payload() {
        let action = EchoAction;
        let result = action.execute(b"not json".to_vec());
        assert!(matches!(result, Err(AppError::InvalidPayload)));
    }
}
