use std::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;

use tonic::transport::Server;
use tonic::{Request, Response, Status};

use prometheus::Counter;
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions};

pub mod container;

use container::logic::MyDockerService;
use container::stats::MyContainerStatsService;
use docker::docker_service_server::DockerServiceServer;
use stats::container_stats_service_server::ContainerStatsServiceServer;

pub mod stats {
	include!("stats.rs");
}

pub mod docker {
	include!("docker.rs");
}


//#[derive(Default)]
//pub struct MyDockerService {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let addr = "[::1]:50051".parse().unwrap();
	let greeter = MyGreeter::default();

	let docker = Docker::connect_with_local_defaults()?;
	let create_options = CreateContainerOptions::default();
	let container_config = Config::default();

	//let docker_service = MyDockerService::default();
	//let container_stats_service = MyContainerStatsService {};

	println!("Worker listening on {}", addr);

	Server::builder()
		//.add_service(DockerServiceServer::new(docker_service))
		//.add_service(ContainerStatsServiceServer::new(container_stats_service))
		.serve(addr)
		.await?;

	Ok(())
}
