use std::sync::{Arc, Mutex};

use clap::Parser;
use platform_manager_app::AppService;
use platform_manager_core::{
    actions::{
        deploy::DeployAction,
        info::InfoAction,
        launched_apps::LaunchedApps,
    },
    models::ApplicationAccess,
};
use platform_manager_grpc::{
    GrpcFactoryService, GrpcInfoService, GrpcLifeCycleService,
};
use std::net::SocketAddr;
use tokio::sync::oneshot;
use tonic::transport::Server;
use tracing::info;

#[derive(Parser)]
#[command(name = "platform_manager_server", about = "Platform Manager gRPC server")]
struct Args {
    #[arg(long, default_value = "[::1]:50051")]
    listen: SocketAddr,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let listen_addr = args.listen;
    let public_url = format!("http://{}", listen_addr);

    let launched_apps = Arc::new(LaunchedApps::new(vec![ApplicationAccess {
        application: "platform-manager".to_string(),
        url: public_url,
    }]));

    let info_action = InfoAction::new(Arc::clone(&launched_apps));
    let deploy_action = DeployAction::new(launched_apps);
    let app = Arc::new(AppService::new(info_action, deploy_action));
    let grpc_info_service = GrpcInfoService::new(Arc::clone(&app));
    let grpc_factory_service = GrpcFactoryService::new(app);
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let grpc_lifecycle_service = GrpcLifeCycleService::new(Arc::new(Mutex::new(Some(shutdown_tx))));

    info!("Starting gRPC server on {}", listen_addr);

    Server::builder()
        .add_service(grpc_info_service.into_server())
        .add_service(grpc_factory_service.into_server())
        .add_service(grpc_lifecycle_service.into_server())
        .serve_with_shutdown(listen_addr, async move {
            let _ = shutdown_rx.await;
            info!("Shutdown signal received via LifeCycle.Terminate");
        })
        .await?;

    info!("Server shut down");
    Ok(())
}

