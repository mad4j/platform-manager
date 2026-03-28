use my_app_app::AppService;
use my_app_app::errors::AppError;
use my_app_transport::{
    ActionRequest, ActionResponse, ActionService, ActionServiceServer, InfoRequest, InfoResponse,
};
use my_app_transport::{from_proto, to_info_proto, to_proto};
use tonic::{Request, Response, Status};
use tracing::info;

pub struct GrpcActionService {
    app: AppService,
}

impl GrpcActionService {
    pub fn new(app: AppService) -> Self {
        Self { app }
    }

    pub fn into_server(self) -> ActionServiceServer<GrpcActionService> {
        ActionServiceServer::new(self)
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
