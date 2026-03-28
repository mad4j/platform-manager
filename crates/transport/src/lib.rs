pub mod action {
    tonic::include_proto!("action");
}

pub mod manager {
    tonic::include_proto!("manager");
}
pub mod mapper;

pub use mapper::{from_proto, to_info_proto, to_proto};
pub use action::{ActionRequest, ActionResponse};
pub use action::action_service_client::ActionServiceClient;
pub use action::action_service_server::{ActionService, ActionServiceServer};
pub use manager::{Endpoint, InfoRequest, InfoResponse};
pub use manager::info_service_client::InfoServiceClient;
pub use manager::info_service_server::{InfoService, InfoServiceServer};
