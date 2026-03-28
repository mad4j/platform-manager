use std::sync::Arc;

use my_app_app::AppService;
use my_app_core::{
    ActionRegistry,
    actions::{echo::EchoAction, info::InfoAction},
};
use my_app_grpc::{GrpcActionService, GrpcInfoService};
use my_app_transport::{ActionRequest, ActionServiceClient, InfoRequest, InfoServiceClient};
use tokio::net::TcpListener;
use tonic::transport::Server;

#[tokio::test]
async fn test_grpc_echo() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let mut registry = ActionRegistry::new();
    registry.register(Box::new(EchoAction));
    registry.register(Box::new(InfoAction));
    let app = Arc::new(AppService::new(registry));
    let grpc_action_service = GrpcActionService::new(Arc::clone(&app));
    let grpc_info_service = GrpcInfoService::new(app);

    let server = Server::builder()
        .add_service(grpc_action_service.into_server())
        .add_service(grpc_info_service.into_server())
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

    let mut registry = ActionRegistry::new();
    registry.register(Box::new(EchoAction));
    registry.register(Box::new(InfoAction));
    let app = Arc::new(AppService::new(registry));
    let grpc_action_service = GrpcActionService::new(Arc::clone(&app));
    let grpc_info_service = GrpcInfoService::new(app);

    let server = Server::builder()
        .add_service(grpc_action_service.into_server())
        .add_service(grpc_info_service.into_server())
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
    assert!(resp.task_id.starts_with("task-"));
}

#[tokio::test]
async fn test_grpc_execute_info_is_rejected() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let mut registry = ActionRegistry::new();
    registry.register(Box::new(EchoAction));
    registry.register(Box::new(InfoAction));
    let app = Arc::new(AppService::new(registry));
    let grpc_action_service = GrpcActionService::new(Arc::clone(&app));
    let grpc_info_service = GrpcInfoService::new(app);

    let server = Server::builder()
        .add_service(grpc_action_service.into_server())
        .add_service(grpc_info_service.into_server())
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
