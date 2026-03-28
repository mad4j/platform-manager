use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InfoEndpoint {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplicationAccess {
    pub application: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfoResponse {
    pub application: String,
    pub endpoints: Vec<InfoEndpoint>,
    pub launched_applications: Vec<ApplicationAccess>,
    pub task_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployAgentRequest {
    pub application: String,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployAgentResponse {
    pub application: String,
    pub url: String,
    pub status: String,
    pub message: String,
}
