use std::sync::Arc;

use my_app_app::AppService;
use my_app_core::{
    ActionRegistry,
    actions::{echo::EchoAction, info::InfoAction},
};
use my_app_grpc::{GrpcActionService, GrpcInfoService};
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let mut registry = ActionRegistry::new();
    registry.register(Box::new(EchoAction));
    registry.register(Box::new(InfoAction));
    let app = Arc::new(AppService::new(registry));
    let grpc_action_service = GrpcActionService::new(Arc::clone(&app));
    let grpc_info_service = GrpcInfoService::new(app);

    let addr = "[::1]:50051".parse()?;
    info!("Starting gRPC server on {}", addr);

    Server::builder()
        .add_service(grpc_action_service.into_server())
        .add_service(grpc_info_service.into_server())
        .serve(addr)
        .await?;

    info!("Server shut down");
    Ok(())
}
