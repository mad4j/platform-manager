use std::sync::{Arc, Mutex};

use my_app_app::AppService;
use my_app_core::{
    ActionRegistry,
    actions::{
        deploy_agent::DeployAgentAction,
        echo::EchoAction,
        info::InfoAction,
        launched_apps::LaunchedApps,
    },
    models::ApplicationAccess,
};
use my_app_grpc::{
    GrpcActionService, GrpcFactoryService, GrpcInfoService, GrpcLifeCycleService,
};
use tokio::sync::oneshot;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let launched_apps = Arc::new(LaunchedApps::new(vec![ApplicationAccess {
        application: "platform-manager".to_string(),
        url: "http://localhost:50051".to_string(),
    }]));

    let mut registry = ActionRegistry::new();
    registry.register(Box::new(EchoAction));
    registry.register(Box::new(DeployAgentAction::new(Arc::clone(&launched_apps))));
    registry.register(Box::new(InfoAction::new(launched_apps)));
    let app = Arc::new(AppService::new(registry));
    let grpc_action_service = GrpcActionService::new(Arc::clone(&app));
    let grpc_info_service = GrpcInfoService::new(Arc::clone(&app));
    let grpc_factory_service = GrpcFactoryService::new(app);
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let grpc_lifecycle_service = GrpcLifeCycleService::new(Arc::new(Mutex::new(Some(shutdown_tx))));

    let addr = "[::1]:50051".parse()?;
    info!("Starting gRPC server on {}", addr);

    Server::builder()
        .add_service(grpc_action_service.into_server())
        .add_service(grpc_info_service.into_server())
        .add_service(grpc_factory_service.into_server())
        .add_service(grpc_lifecycle_service.into_server())
        .serve_with_shutdown(addr, async move {
            let _ = shutdown_rx.await;
            info!("Shutdown signal received via LifeCycle.Terminate");
        })
        .await?;

    info!("Server shut down");
    Ok(())
}
