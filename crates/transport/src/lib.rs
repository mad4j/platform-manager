pub mod action {
    tonic::include_proto!("action");
}

pub mod manager {
    tonic::include_proto!("manager");
}

pub mod factory {
    tonic::include_proto!("factory");
}

pub mod mapper;

pub use mapper::{from_proto, to_info_proto, to_proto};
pub use action::{ActionRequest, ActionResponse};
pub use action::action_service_client::ActionServiceClient;
pub use action::action_service_server::{ActionService, ActionServiceServer};
pub use manager::{Endpoint, InfoRequest, InfoResponse, TerminateRequest, TerminateResponse};
pub use manager::info_service_client::InfoServiceClient;
pub use manager::info_service_server::{InfoService, InfoServiceServer};
pub use manager::life_cycle_client::LifeCycleClient;
pub use manager::life_cycle_server::{LifeCycle, LifeCycleServer};
pub use factory::{DeployAgentRequest, DeployAgentResponse};
pub use factory::factory_service_client::FactoryServiceClient;
pub use factory::factory_service_server::{FactoryService, FactoryServiceServer};
