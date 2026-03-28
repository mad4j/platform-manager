use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EchoRequest {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EchoResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfoEndpoint {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfoResponse {
    pub application: String,
    pub endpoints: Vec<InfoEndpoint>,
    pub task_id: String,
}
