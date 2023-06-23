use futures::StreamExt;
use prometheus::{TextEncoder, Encoder};

use serde::Deserialize;
use tokio::time::{sleep, Duration};
use std::env;
use std::error::Error;
use shiplift::{Docker, ContainerListOptions, ContainerOptions};

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

async fn track_container_stats() -> Result<(), Box<dyn Error>> {
	let docker = Docker::new();
	let options = ContainerListOptions::builder()
		.all()
		.build();
	loop {
		let containers = docker.containers().list(&options).await?;

		let mut cpu_usage_sum = 0.0;
		let mut memory_usage_sum = 0.0;
		let mut _container_count = 0;

		for container in &containers {
			let mut stats_stream = docker.containers().get(&container.id).stats();
			
			while let Some(stats_result) = stats_stream.next().await {
                let stats = stats_result?;

                let cpu_stats = stats.cpu_stats;
                let cpu_percent = calculate_cpu_percent(&cpu_stats);
                println!("Container ID: {}", container.id);
                println!("CPU Usage: {}%", cpu_percent);
                cpu_usage_sum += cpu_stats.cpu_usage.total_usage as f64;
                
                
                let memory_stats = stats.memory_stats;
                if let Some(failcnt) = memory_stats.failcnt {
                    println!("Memory fail count: {}", failcnt);
                }
                let memory_usage = memory_stats.usage as f64;
                println!("Memory usage: {} bytes", memory_usage);
                memory_usage_sum += memory_usage;

				_container_count += 1;
            }
        
        }
        
		let avg_cpu_usage = cpu_usage_sum / containers.len() as f64;
		let avg_memory_usage = memory_usage_sum / containers.len() as f64;

		println!("Average CPU usage: {}%", avg_cpu_usage);
		println!("Average memory usage: {} bytes", avg_memory_usage);

		if avg_cpu_usage > 70.0 || avg_memory_usage > 100_000_000.0 {
			scale().await;
		}

		sleep(Duration::from_secs(60)).await;
	}
}

async fn scale() -> Result<impl warp::Reply, warp::Rejection> {
    for _ in 0..CONTAINER_THRESHOLD {
        // atm im not sure what image we should be using here
        create_container("ubuntu:latest").await.map_err(|_| warp::reject::not_found())?;
    }
    Ok(format!("Successfully scaled up containers by {}", CONTAINER_THRESHOLD))
}

fn calculate_cpu_percent(cpu_stats: &shiplift::rep::CpuStats) -> f64 {
	let cpu_delta = cpu_stats.cpu_usage.total_usage - cpu_stats.cpu_usage.usage_in_kernelmode;
    let system_delta = cpu_stats.system_cpu_usage - cpu_stats.cpu_usage.usage_in_kernelmode;

	if system_delta > 0 && cpu_delta > 0 {
		let cpu_percent = (cpu_delta as f64 / system_delta as f64) * 100.0;
		cpu_percent.round()
	} else {
		0.0
	}
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
async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
	if err.is_not_found() {
		return Ok(warp::reply::with_status(
			"not found",
			warp::http::StatusCode::NOT_FOUND,
		));
	}

	eprintln!("unhandled rejection: {:?}", err);
	Ok(warp::reply::with_status(
		"internal server error",
		warp::http::StatusCode::INTERNAL_SERVER_ERROR,
	))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let _prometheus_addr = env::var("PROMETHEUS_ADDR").unwrap_or_else(|_| PROMETHEUS_ADDR.to_string());
	let prometheus_metrics = warp::path("metrics").and_then(|| async {
		let encoder = TextEncoder::new();
		let metrics_families = prometheus::gather();
		let mut buffer = vec![];
		encoder.encode(&metrics_families, &mut buffer).unwrap();
		Ok::<_, std::convert::Infallible>(
			Response::builder()
				.header("Content-Type", encoder.format_type())
				.body(buffer)
		)
	});

	track_container_stats().await?;
	
	// let rate_limiter = Arc::new(DirectRateLimiter::<GCRA>::per_second(std::num::NonZeroU32::new(10).unwrap()));
	let health_check_route = warp::path("health")
        .and(warp::get())
        .map(warp::reply);

	let hello_route = warp::path("hello")
    	.and(warp::get())
    	.and(warp::path::end())
    	.and(warp::any().map(|| "Hello, World!"));

	let scale_route = warp::path("scale")
		.and(warp::get())
		.and(warp::path::end())
		.and_then(|| async { scale().await })
	 	.recover(handle_rejection);
     
	let routes = health_check_route
		.or(hello_route)
		.or(scale_route);

	let server_address = ([127, 0, 0, 1], 8087);
    println!("Scaler server running at http://localhost:8087");

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