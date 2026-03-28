use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

use my_app_app::AppService;
use my_app_app::errors::AppError;
use my_app_transport::{
    ActionRequest, ActionResponse, ActionService, ActionServiceServer, DeployAgentRequest,
    DeployAgentResponse, FactoryService, FactoryServiceServer, InfoRequest, InfoResponse,
    InfoService, InfoServiceServer, LifeCycle, LifeCycleServer, TerminateRequest,
    TerminateResponse,
};
use my_app_transport::{from_proto, to_info_proto, to_proto};
use tonic::{Request, Response, Status};
use tracing::info;

pub struct GrpcActionService {
    app: Arc<AppService>,
}

pub struct GrpcInfoService {
    app: Arc<AppService>,
}

pub struct GrpcLifeCycleService {
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

pub struct GrpcFactoryService {
    app: Arc<AppService>,
}

impl GrpcActionService {
    pub fn new(app: Arc<AppService>) -> Self {
        Self { app }
    }

    pub fn into_server(self) -> ActionServiceServer<GrpcActionService> {
        ActionServiceServer::new(self)
    }
}

impl GrpcInfoService {
    pub fn new(app: Arc<AppService>) -> Self {
        Self { app }
    }

    pub fn into_server(self) -> InfoServiceServer<GrpcInfoService> {
        InfoServiceServer::new(self)
    }
}

impl GrpcLifeCycleService {
    pub fn new(shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>) -> Self {
        Self { shutdown_tx }
    }

    pub fn into_server(self) -> LifeCycleServer<GrpcLifeCycleService> {
        LifeCycleServer::new(self)
    }
}

impl GrpcFactoryService {
    pub fn new(app: Arc<AppService>) -> Self {
        Self { app }
    }

    pub fn into_server(self) -> FactoryServiceServer<GrpcFactoryService> {
        FactoryServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl ActionService for GrpcActionService {
    async fn execute(
        &self,
        request: Request<ActionRequest>,
    ) -> Result<Response<ActionResponse>, Status> {
        let req = request.into_inner();
        info!(action = %req.action, "received gRPC request");
        if req.action == "info" {
            let response = to_proto(Err(AppError::Execution(
                "action 'info' is available only via Info RPC".to_string(),
            )));
            return Ok(Response::new(response));
        }
        let (action_name, payload) = from_proto(req);
        let result = self.app.execute(&action_name, payload);
        let response = to_proto(result);
        Ok(Response::new(response))
    }
}

#[tonic::async_trait]
impl InfoService for GrpcInfoService {
    async fn info(
        &self,
        _request: Request<InfoRequest>,
    ) -> Result<Response<InfoResponse>, Status> {
        info!("received gRPC info request");
        let result = self.app.execute("info", vec![]);
        let response = to_info_proto(result);
        Ok(Response::new(response))
    }
}

#[tonic::async_trait]
impl LifeCycle for GrpcLifeCycleService {
    async fn terminate(
        &self,
        _request: Request<TerminateRequest>,
    ) -> Result<Response<TerminateResponse>, Status> {
        info!("received gRPC lifecycle terminate request");

        let tx = self
            .shutdown_tx
            .lock()
            .map_err(|_| Status::internal("failed to acquire shutdown lock"))?
            .take();

        let message = if let Some(tx) = tx {
            let _ = tx.send(());
            "termination requested".to_string()
        } else {
            "termination already requested".to_string()
        };

        Ok(Response::new(TerminateResponse { message }))
    }
}

#[tonic::async_trait]
impl FactoryService for GrpcFactoryService {
    async fn deploy_agent(
        &self,
        request: Request<DeployAgentRequest>,
    ) -> Result<Response<DeployAgentResponse>, Status> {
        info!("received gRPC factory deploy-agent request");

        let req = request.into_inner();
        let result = self
            .app
            .execute("deploy-agent", req.config.into_bytes());

        match result {
            Ok(payload) => {
                let value: serde_json::Value = serde_json::from_slice(&payload).map_err(|e| {
                    Status::internal(format!("failed to parse deploy-agent response: {e}"))
                })?;

                let agent_id = value
                    .get("agent_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let message = value
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| {
                        if agent_id.is_empty() {
                            "deploy-agent completed"
                        } else {
                            "agent deployed"
                        }
                    })
                    .to_string();

                Ok(Response::new(DeployAgentResponse {
                    agent_id,
                    message,
                    error: String::new(),
                }))
            }
            Err(err) => Ok(Response::new(DeployAgentResponse {
                agent_id: String::new(),
                message: String::new(),
                error: err.to_string(),
            })),
        }
    }
}
