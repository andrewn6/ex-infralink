use prometheus::{TextEncoder, Encoder};

use reqwest::{Error};
use serde::{Deserialize};
use std::env;
use std::net::SocketAddr;

use warp::{Filter, reject};
use warp::http::Response;
use dotenv_codegen::dotenv;

const VULTR_API_KEY: &str = dotenv!("VULTR_API_KEY");

const PROMETHEUS_ADDR: &str = dotenv!("PROMETHEUS_ADDR");

const CONTAINER_THRESHOLD: usize = 2; // Number of containers to create when scaling

#[derive(Debug, Deserialize)]
struct Instance {
	id: String,
	name: String,
	// ... add rest
}

#[derive(Debug)]
struct InvalidParameter;

impl reject::Reject for InvalidParameter {}

#[derive(Debug, Deserialize)]
struct CreateInstanceResponse {
	instance: Instance,
}

#[derive(Debug, Deserialize)]
struct CreateInstancePayload {
	region: String,
	plan: String,
}

async fn create_container(image: &str) -> Result<(), Box<dyn Error>> {
	let docker = Docker::new();

	let options = ContainerOptions::builder(image)
		.auto_remove(true)
		.build();
	
	docker.containers().create(&options).await?;

	Ok(())
}

async fn delete_container(container_id: &str) -> Result<(), Box<dyn Error>> {
	let docker = Docker::new();

    docker.containers()
        .get(container_id)
        .delete()
        .await?;

    Ok(())	
}

#[tokio::main]
async fn main() -> Result<(), Error> {
	let _prometheus_addr = env::var("PROMETHEUS_ADDR").unwrap_or_else(|_| PROMETHEUS_ADDR.to_string());
	let prometheus_metrics = warp::path("metrics").map(|| {
		let encoder = TextEncoder::new();
		let metrics_families = prometheus::gather();
		let mut buffer = vec![];
		encoder.encode(&metrics_families, &mut buffer).unwrap();
		Response::builder()
			.header("Content-Type", encoder.format_type())
			.body(buffer)
	});
	
	// let rate_limiter = Arc::new(DirectRateLimiter::<GCRA>::per_second(std::num::NonZeroU32::new(10).unwrap()));
	let health_check_route = warp::path("health")
        .and(warp::get())
        .map(warp::reply);
	let hello_route = warp::path("hello")
    	.and(warp::get())
    	.and(warp::path::end())
    	.and(warp::any().map(|| "Hello, World!"));

	let routes = health_check_route
		.or(hello_route)
		.or(prometheus_metrics);

	let server_address = ([127, 0, 0, 1], 8087);
    println!("Server running at http://localhost:8087");

    warp::serve(routes).run(server_address).await;

	Ok(())
}

/* 
fn with_rate_limit(
    rate_limiter: Arc<DirectRateLimiter<GCRA>>,
)  {
    warp::any()
        .and_then(move || {
            let rate_limiter = rate_limiter.clone();
            async move {
                if rate_limiter.check().is_err() {
                    return Err(warp::reject::custom(InvalidParameter));
                }
                Ok(())
            }
        });
}
*/