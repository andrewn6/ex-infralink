mod rollout {
	include!("../rollout.rs");
}

use rollout::{BlueGreen, BlueGreenRequest, DeployResponse};

use tonic::{transport::Server, Request, Response, Status};
use std::net::SocketAddr;
use bollard::Docker;
use bollard::models::{ContainerCreateResponse, HostConfig, ContainerListOptions};

pub mod strategies;

#[derive(Debug)]
pub struct BlueGreenService {
	docker: Docker,
}

#[tonic::async_trait]
impl BlueGreen for BlueGreenRollout {
	async fn deploy(&self, request: Request<BlueGreenRequest>) -> Result<Response<DeployResponse>, Status> {
		todo!()
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let addr = "127.0.0.1:50052".parse().unwrap();
	let blue_green_service = BlueGreenRollout::default();

	println("Rollout server listening on {}", addr);

	Server::builder()
        .add_service(BlueGreen::new(blue_green_service))
        .serve(addr)
        .await?;

    Ok(())
}