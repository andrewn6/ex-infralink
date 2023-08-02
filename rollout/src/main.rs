mod rollout {
	include!("./rollout.rs");
}

use tonic::{transport::Server, Request, Response, Status};
use std::net::SocketAddr;
use bollard::Docker;
use bollard::models::{ContainerCreateResponse, HostConfig};

pub mod strategies;

#[derive(Debug)]
pub struct BlueGreenService {
	docker: Docker,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let addr = "127.0.0.1:50052".parse().unwrap();

	println!("Rollout server listening on {}", addr);

    Ok(())
}