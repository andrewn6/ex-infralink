mod rollout {
    include!("../rollout.rs");
}

use std::net::SocketAddr;
use tonic::{Request, Response, Status};
use rollout::{RolloutStrategy, StrategyType};
use rollout::rollout_strategy::Rollout;
use surf::{Client, http::Url};

const OLD_APP_ADDR: &str = "http://127.0.0.1:7000"; // Replace with the actual address of the old version
const NEW_APP_ADDR: &str = "http://127.0.0.1:7001";

#[derive(Default)]
pub struct BlueGreenRollout {}

#[tonic::async_trait]
impl BlueGreen for BlueGreenService {
    async fn deploy(
        &self,
        request: Request<BlueGreenRequest>,
    ) -> Result<Response<DeployResponse>, Status> {
        todo!()
    }
}