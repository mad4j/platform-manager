use std::sync::Mutex;
use std::sync::Arc;

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
use my_app_grpc::{GrpcActionService, GrpcInfoService, GrpcLifeCycleService};
use my_app_transport::{
    ActionRequest, ActionServiceClient, InfoRequest, InfoServiceClient, LifeCycleClient,
    TerminateRequest,
};
use tokio::sync::oneshot;
use tokio::net::TcpListener;
use tonic::transport::Server;

fn build_registry() -> ActionRegistry {
    let launched_apps = Arc::new(LaunchedApps::new(vec![ApplicationAccess {
        application: "platform-manager".to_string(),
        url: "http://localhost:50051".to_string(),
    }]));

    let mut registry = ActionRegistry::new();
    registry.register(Box::new(EchoAction));
    registry.register(Box::new(DeployAgentAction::new(Arc::clone(&launched_apps))));
    registry.register(Box::new(InfoAction::new(launched_apps)));
    registry
}

#[tokio::test]
async fn test_grpc_echo() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let registry = build_registry();
    let app = Arc::new(AppService::new(registry));
    let grpc_action_service = GrpcActionService::new(Arc::clone(&app));
    let grpc_info_service = GrpcInfoService::new(app);
    let (shutdown_tx, _shutdown_rx) = oneshot::channel::<()>();
    let grpc_lifecycle_service =
        GrpcLifeCycleService::new(Arc::new(Mutex::new(Some(shutdown_tx))));

    let server = Server::builder()
        .add_service(grpc_action_service.into_server())
        .add_service(grpc_info_service.into_server())
        .add_service(grpc_lifecycle_service.into_server())
        .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener));

    tokio::spawn(server);

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let endpoint = format!("http://{}", addr);
    let channel = tonic::transport::Channel::from_shared(endpoint).unwrap().connect().await.unwrap();
    let mut client = ActionServiceClient::new(channel);

    let payload = serde_json::json!({"message": "hello"}).to_string().into_bytes();
    let request = tonic::Request::new(ActionRequest {
        action: "echo".to_string(),
        payload,
    });

    let response = client.execute(request).await.unwrap();
    let resp = response.into_inner();
    assert!(resp.error.is_empty());
    let val: serde_json::Value = serde_json::from_slice(&resp.payload).unwrap();
    assert_eq!(val["message"], "hello");
}

#[tokio::test]
async fn test_grpc_info() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let registry = build_registry();
    let app = Arc::new(AppService::new(registry));
    let grpc_action_service = GrpcActionService::new(Arc::clone(&app));
    let grpc_info_service = GrpcInfoService::new(app);
    let (shutdown_tx, _shutdown_rx) = oneshot::channel::<()>();
    let grpc_lifecycle_service =
        GrpcLifeCycleService::new(Arc::new(Mutex::new(Some(shutdown_tx))));

    let server = Server::builder()
        .add_service(grpc_action_service.into_server())
        .add_service(grpc_info_service.into_server())
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
    assert_eq!(resp.endpoints.len(), 2);
    assert_eq!(resp.launched_applications.len(), 1);
    assert_eq!(resp.launched_applications[0].application, "platform-manager");
    assert_eq!(resp.launched_applications[0].url, "http://localhost:50051");
    assert!(resp.task_id.starts_with("task-"));
}

#[tokio::test]
async fn test_grpc_execute_info_is_rejected() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let registry = build_registry();
    let app = Arc::new(AppService::new(registry));
    let grpc_action_service = GrpcActionService::new(Arc::clone(&app));
    let grpc_info_service = GrpcInfoService::new(app);
    let (shutdown_tx, _shutdown_rx) = oneshot::channel::<()>();
    let grpc_lifecycle_service =
        GrpcLifeCycleService::new(Arc::new(Mutex::new(Some(shutdown_tx))));

    let server = Server::builder()
        .add_service(grpc_action_service.into_server())
        .add_service(grpc_info_service.into_server())
        .add_service(grpc_lifecycle_service.into_server())
        .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener));

    tokio::spawn(server);

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let endpoint = format!("http://{}", addr);
    let channel = tonic::transport::Channel::from_shared(endpoint).unwrap().connect().await.unwrap();
    let mut client = ActionServiceClient::new(channel);

    let request = tonic::Request::new(ActionRequest {
        action: "info".to_string(),
        payload: vec![],
    });
    let response = client.execute(request).await.unwrap();
    let resp = response.into_inner();
    assert!(resp.payload.is_empty());
    assert!(resp.error.contains("only via Info RPC"));
}

#[tokio::test]
async fn test_grpc_deploy_agent_updates_info_report() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let registry = build_registry();
    let app = Arc::new(AppService::new(registry));
    let grpc_action_service = GrpcActionService::new(Arc::clone(&app));
    let grpc_info_service = GrpcInfoService::new(app);
    let (shutdown_tx, _shutdown_rx) = oneshot::channel::<()>();
    let grpc_lifecycle_service =
        GrpcLifeCycleService::new(Arc::new(Mutex::new(Some(shutdown_tx))));

    let server = Server::builder()
        .add_service(grpc_action_service.into_server())
        .add_service(grpc_info_service.into_server())
        .add_service(grpc_lifecycle_service.into_server())
        .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener));

    tokio::spawn(server);

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let endpoint = format!("http://{}", addr);
    let action_channel = tonic::transport::Channel::from_shared(endpoint.clone())
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut action_client = ActionServiceClient::new(action_channel);

    let deploy_payload = serde_json::json!({
        "application": "orders-api",
        "url": "https://orders.example.com"
    })
        .to_string()
        .into_bytes();
    let deploy_request = tonic::Request::new(ActionRequest {
        action: "deploy-agent".to_string(),
        payload: deploy_payload,
    });
    let deploy_response = action_client.execute(deploy_request).await.unwrap();
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

    let registry = build_registry();
    let app = Arc::new(AppService::new(registry));
    let grpc_action_service = GrpcActionService::new(Arc::clone(&app));
    let grpc_info_service = GrpcInfoService::new(app);
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let grpc_lifecycle_service =
        GrpcLifeCycleService::new(Arc::new(Mutex::new(Some(shutdown_tx))));

    let server = Server::builder()
        .add_service(grpc_action_service.into_server())
        .add_service(grpc_info_service.into_server())
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
