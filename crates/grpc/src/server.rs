use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

use platform_manager_app::AppService;
use platform_manager_transport::{
    DeployRequest, DeployResponse, FactoryService, FactoryServiceServer,
    InfoRequest, InfoResponse, InfoService, InfoServiceServer,
    LifeCycle, LifeCycleServer, TerminateRequest, TerminateResponse,
};
use platform_manager_transport::to_info_proto;
use tonic::{Request, Response, Status};
use tracing::info;

pub struct GrpcInfoService {
    app: Arc<AppService>,
}

pub struct GrpcLifeCycleService {
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

pub struct GrpcFactoryService {
    app: Arc<AppService>,
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
impl InfoService for GrpcInfoService {
    async fn info(
        &self,
        _request: Request<InfoRequest>,
    ) -> Result<Response<InfoResponse>, Status> {
        info!("received gRPC info request");
        let result = self.app.get_info();
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
    async fn deploy(
        &self,
        request: Request<DeployRequest>,
    ) -> Result<Response<DeployResponse>, Status> {
        info!("received gRPC factory deploy request");

        let req = request.into_inner();
        let result = self.app.deploy(req.config.into_bytes());

        match result {
            Ok(payload) => {
                let value: serde_json::Value = serde_json::from_slice(&payload).map_err(|e| {
                    Status::internal(format!("failed to parse deploy response: {e}"))
                })?;

                let application_id = value
                    .get("application_id")
                    .or_else(|| value.get("agent_id"))
                    .or_else(|| value.get("application"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let message = value
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| {
                        if application_id.is_empty() {
                            "deploy completed"
                        } else {
                            "application deployed"
                        }
                    })
                    .to_string();

                Ok(Response::new(DeployResponse {
                    agent_id: application_id.clone(),
                    message,
                    error: String::new(),
                    application_id,
                }))
            }
            Err(err) => Ok(Response::new(DeployResponse {
                agent_id: String::new(),
                message: String::new(),
                error: err.to_string(),
                application_id: String::new(),
            })),
        }
    }
}

