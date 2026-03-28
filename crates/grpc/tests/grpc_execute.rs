use std::sync::Mutex;
use std::sync::Arc;

use my_app_app::AppService;
use my_app_core::{
    actions::{
        deploy_agent::DeployAgentAction,
        info::InfoAction,
        launched_apps::LaunchedApps,
    },
    models::ApplicationAccess,
};
use my_app_grpc::{GrpcFactoryService, GrpcInfoService, GrpcLifeCycleService};
use my_app_transport::{
    DeployAgentRequest, FactoryServiceClient, InfoRequest, InfoServiceClient,
    LifeCycleClient, TerminateRequest,
};
use tokio::sync::oneshot;
use tokio::net::TcpListener;
use tonic::transport::Server;

fn build_app_service() -> AppService {
    let launched_apps = Arc::new(LaunchedApps::new(vec![ApplicationAccess {
        application: "platform-manager".to_string(),
        url: "http://localhost:50051".to_string(),
    }]));

    AppService::new(
        InfoAction::new(Arc::clone(&launched_apps)),
        DeployAgentAction::new(launched_apps),
    )
}

#[tokio::test]
async fn test_grpc_info() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let app = Arc::new(build_app_service());
    let grpc_info_service = GrpcInfoService::new(Arc::clone(&app));
    let grpc_factory_service = GrpcFactoryService::new(app);
    let (shutdown_tx, _shutdown_rx) = oneshot::channel::<()>();
    let grpc_lifecycle_service =
        GrpcLifeCycleService::new(Arc::new(Mutex::new(Some(shutdown_tx))));

    let server = Server::builder()
        .add_service(grpc_info_service.into_server())
        .add_service(grpc_factory_service.into_server())
        .add_service(grpc_lifecycle_service.into_server())
        .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener));

    tokio::spawn(server);

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let endpoint = format!("http://{}", addr);
    let channel = tonic::transport::Channel::from_shared(endpoint).unwrap().connect().await.unwrap();
    let mut client = InfoServiceClient::new(channel);

    let request = tonic::Request::new(InfoRequest {});
    let response = client.info(request).await.unwrap();
    let resp = response.into_inner();
    assert!(resp.error.is_empty());
    assert_eq!(resp.application, "platform-manager");
    assert_eq!(resp.endpoints.len(), 1);
    assert_eq!(resp.launched_applications.len(), 1);
    assert_eq!(resp.launched_applications[0].application, "platform-manager");
    assert_eq!(resp.launched_applications[0].url, "http://localhost:50051");
    assert!(resp.task_id.starts_with("task-"));
}

#[tokio::test]
async fn test_grpc_deploy_agent_updates_info_report() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let app = Arc::new(build_app_service());
    let grpc_info_service = GrpcInfoService::new(Arc::clone(&app));
    let grpc_factory_service = GrpcFactoryService::new(Arc::clone(&app));
    let (shutdown_tx, _shutdown_rx) = oneshot::channel::<()>();
    let grpc_lifecycle_service =
        GrpcLifeCycleService::new(Arc::new(Mutex::new(Some(shutdown_tx))));

    let server = Server::builder()
        .add_service(grpc_info_service.into_server())
        .add_service(grpc_factory_service.into_server())
        .add_service(grpc_lifecycle_service.into_server())
        .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener));

    tokio::spawn(server);

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let endpoint = format!("http://{}", addr);
    let factory_channel = tonic::transport::Channel::from_shared(endpoint.clone())
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut factory_client = FactoryServiceClient::new(factory_channel);

    let config = serde_json::json!({
        "application": "orders-api",
        "url": "https://orders.example.com"
    })
    .to_string();
    let deploy_request = tonic::Request::new(DeployAgentRequest { config });
    let deploy_response = factory_client.deploy_agent(deploy_request).await.unwrap();
    let deploy_result = deploy_response.into_inner();
    assert!(deploy_result.error.is_empty());

    let info_channel = tonic::transport::Channel::from_shared(endpoint)
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut info_client = InfoServiceClient::new(info_channel);

    let info_response = info_client
        .info(tonic::Request::new(InfoRequest {}))
        .await
        .unwrap();
    let info = info_response.into_inner();
    assert!(info.error.is_empty());
    assert_eq!(
        info.launched_applications
            .iter()
            .map(|app| (app.application.clone(), app.url.clone()))
            .collect::<Vec<_>>(),
        vec![
            ("platform-manager".to_string(), "http://localhost:50051".to_string()),
            ("orders-api".to_string(), "https://orders.example.com".to_string())
        ]
    );
}

#[tokio::test]
async fn test_grpc_terminate_requests_shutdown() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let app = Arc::new(build_app_service());
    let grpc_info_service = GrpcInfoService::new(Arc::clone(&app));
    let grpc_factory_service = GrpcFactoryService::new(app);
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let grpc_lifecycle_service =
        GrpcLifeCycleService::new(Arc::new(Mutex::new(Some(shutdown_tx))));

    let server = Server::builder()
        .add_service(grpc_info_service.into_server())
        .add_service(grpc_factory_service.into_server())
        .add_service(grpc_lifecycle_service.into_server())
        .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener));

    tokio::spawn(server);

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let endpoint = format!("http://{}", addr);
    let channel = tonic::transport::Channel::from_shared(endpoint)
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut lifecycle_client = LifeCycleClient::new(channel);

    let response = lifecycle_client
        .terminate(tonic::Request::new(TerminateRequest {}))
        .await
        .unwrap();
    let resp = response.into_inner();
    assert_eq!(resp.message, "termination requested");

    tokio::time::timeout(std::time::Duration::from_secs(1), shutdown_rx)
        .await
        .unwrap()
        .unwrap();
}
