use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EchoRequest {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EchoResponse {
    pub message: String,
}
