use my_app_app::AppService;
use my_app_transport::{ActionRequest, ActionResponse, ActionService, ActionServiceServer};
use my_app_transport::{from_proto, to_proto};
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
        let (action_name, payload) = from_proto(req);
        let result = self.app.execute(&action_name, payload);
        let response = to_proto(result);
        Ok(Response::new(response))
    }
}
