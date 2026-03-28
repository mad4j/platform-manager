use my_app_app::AppService;
use my_app_core::{
    ActionRegistry,
    actions::{echo::EchoAction, info::InfoAction},
};
use my_app_grpc::GrpcActionService;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let mut registry = ActionRegistry::new();
    registry.register(Box::new(EchoAction));
    registry.register(Box::new(InfoAction));
    let app = AppService::new(registry);
    let grpc_service = GrpcActionService::new(app);

    let addr = "[::1]:50051".parse()?;
    info!("Starting gRPC server on {}", addr);

    Server::builder()
        .add_service(grpc_service.into_server())
        .serve(addr)
        .await?;

    info!("Server shut down");
    Ok(())
}
