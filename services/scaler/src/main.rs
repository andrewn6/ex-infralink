use prometheus::{TextEncoder, Encoder};
use ratelimit_meter::{DirectRateLimiter, GCRA};
use reqwest::{Client, Error, header};
use serde::{Deserialize, Serialize};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
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

#[tokio::main]
async fn main() -> Result<(), Error> {
	let prometheus_addr = env::var("PROMETHEUS_ADDR").unwrap_or_else(|_| PROMETHEUS_ADDR.to_string());
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

	let server_port: u16 = 8087;
	let server_address: String = format!("0.0.0.0:{}", server_port);

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

	warp::serve(routes).run(server_address.parse::<SocketAddr>().unwrap());

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