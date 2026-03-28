pub mod manager {
    tonic::include_proto!("manager");
}

pub mod factory {
    tonic::include_proto!("factory");
}

pub mod mapper;

pub use mapper::to_info_proto;
pub use manager::{Endpoint, InfoRequest, InfoResponse, TerminateRequest, TerminateResponse};
pub use manager::info_service_client::InfoServiceClient;
pub use manager::info_service_server::{InfoService, InfoServiceServer};
pub use manager::life_cycle_client::LifeCycleClient;
pub use manager::life_cycle_server::{LifeCycle, LifeCycleServer};
pub use factory::{DeployAgentRequest, DeployAgentResponse};
pub use factory::factory_service_client::FactoryServiceClient;
pub use factory::factory_service_server::{FactoryService, FactoryServiceServer};
