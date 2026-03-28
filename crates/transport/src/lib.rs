pub mod proto {
    tonic::include_proto!("action");
}
pub mod mapper;

pub use mapper::{from_proto, to_info_proto, to_proto};
pub use proto::{ActionRequest, ActionResponse, Endpoint, InfoRequest, InfoResponse};
pub use proto::action_service_client::ActionServiceClient;
pub use proto::action_service_server::{ActionService, ActionServiceServer};
