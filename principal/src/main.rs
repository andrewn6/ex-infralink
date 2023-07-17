pub mod providers;
pub mod shared_config;
use std::convert::Infallible;
use std::net::SocketAddr;

use dotenv::dotenv;

pub mod rules;
pub mod manager;
pub mod db;
pub mod gpu;
pub mod volumes;

use rules::rules::create_manager_rules;

use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

async fn hello_world(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
	Ok(Response::new("Hello, World".into()))
}

#[tokio::main]
async fn main() {
	let addr = SocketAddr::from(([127, 0, 0, 1], 8081));

	let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(hello_world))
    });

	let server = Server::bind(&addr).serve(make_svc);

	// Load environment variables into runtime
	dotenv().unwrap();

	if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
	
	// Only uncomment if changes are made
	match create_manager_rules().await {
		Ok(_) => println!("Data creation successful"),
		Err(err) => eprintln!("Data creation failed: {}", err),
	}

	println!("Principal Server listening on {}", addr.to_string());
}