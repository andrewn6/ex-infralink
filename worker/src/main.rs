use tokio::sync::Mutex;
use std::sync::Arc;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions};

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};

pub mod container;

mod healer {
    pub mod healer;
}

use healer::healer::MyHealer;
use proto_healer::healer_server::HealerServer;

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

pub mod proto_healer {
	include!("healer.rs");
}

mod hello_world {
	include!("helloworld.rs");
}

mod proto_compute {
	include!("compute.rs");
}

mod proto_memory {
	include!("memory.rs");

	pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
		tonic::include_file_descriptor_set!("greeter_descriptor");
}

mod proto_storage {
	include!("storage.rs");
}

mod proto_network {
	include!("network.rs");
}

//#[derive(Default)]
//pub struct MyDockerService {}

#[derive(Default)]
pub struct ComputeServiceImpl {}

#[derive(Default)]
pub struct MemoryServiceImpl {}

#[derive(Default)]
pub struct StorageServiceImpl {}

#[derive(Default)]
pub struct NetworkServiceImpl {}

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
	async fn say_hello(
		&self,
		request: Request<HelloRequest>,
	) -> Result<Response<HelloReply>, Status> {
		println!("Got a request from {:?}", request.remote_addr());

		let reply = hello_world::HelloReply {
			message: format!("Hello {}!", request.into_inner().name),
		};
		Ok(Response::new(reply))
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let addr = "[::1]:50051".parse().unwrap();
	let greeter = MyGreeter::default();

	let docker = Docker::connect_with_local_defaults()?;
	let create_options = CreateContainerOptions::default();
	let container_config = Config::default();

	//let docker_service = MyDockerService::default();
	//let container_stats_service = MyContainerStatsService {};
	let healer = MyHealer {
		docker,
		create_options,
		container_config,
		healing: Arc::new(Mutex::new(false)),
		healing_report: Arc::new(Mutex::new(Vec::new())),
	};

	let reflection_service = tonic_reflection::server::Builder::configure()
		.register_encoded_file_descriptor_set(proto_memory::FILE_DESCRIPTOR_SET)
		.build()
		.unwrap();

	println!("GreeterServer listening on {}", addr);

	Server::builder()
		.add_service(GreeterServer::new(greeter))
		.add_service(reflection_service)
		.add_service(HealerServer::new(healer))
		//.add_service(DockerServiceServer::new(docker_service))
		//.add_service(ContainerStatsServiceServer::new(container_stats_service))
		.serve(addr)
		.await?;

	Ok(())
}
