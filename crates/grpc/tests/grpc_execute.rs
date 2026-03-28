use my_app_app::AppService;
use my_app_core::{ActionRegistry, actions::echo::EchoAction};
use my_app_grpc::GrpcActionService;
use my_app_transport::{ActionRequest, ActionServiceClient};
use tokio::net::TcpListener;
use tonic::transport::Server;

#[tokio::test]
async fn test_grpc_echo() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let mut registry = ActionRegistry::new();
    registry.register(Box::new(EchoAction));
    let app = AppService::new(registry);
    let grpc_service = GrpcActionService::new(app);

    let server = Server::builder()
        .add_service(grpc_service.into_server())
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
