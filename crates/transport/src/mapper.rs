use crate::proto::{ActionRequest, ActionResponse};
use my_app_core::AppError;

pub fn from_proto(req: ActionRequest) -> (String, Vec<u8>) {
    (req.action, req.payload)
}

pub fn to_proto(res: Result<Vec<u8>, AppError>) -> ActionResponse {
    match res {
        Ok(payload) => ActionResponse {
            payload,
            error: String::new(),
        },
        Err(e) => ActionResponse {
            payload: vec![],
            error: e.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_proto() {
        let req = ActionRequest {
            action: "echo".to_string(),
            payload: b"data".to_vec(),
        };
        let (action, payload) = from_proto(req);
        assert_eq!(action, "echo");
        assert_eq!(payload, b"data");
    }

    #[test]
    fn test_to_proto_success() {
        let resp = to_proto(Ok(b"result".to_vec()));
        assert_eq!(resp.payload, b"result");
        assert!(resp.error.is_empty());
    }

    #[test]
    fn test_to_proto_error() {
        let resp = to_proto(Err(AppError::ActionNotFound("test".to_string())));
        assert!(resp.payload.is_empty());
        assert!(!resp.error.is_empty());
    }
}
