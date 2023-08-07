use dotenv_codegen::dotenv;
use models::models::health_check::{HealthCheck, HealthCheckType, HttpMethod, CustomCheckType, CustomHealthCheck};
use models::models::network::Network;
use redis::aio::Connection;
use redis::AsyncCommands;
use sqlx::{PgPool, Error as SqlxError};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::time::sleep;
use uuid::Uuid;

/// Struct to hold information about an ongoing health check.
pub struct HealthCheckTask {
	// The JoinHandle for the health check loop.
	pub handle: tokio::task::JoinHandle<()>,
	// Unique ID of the health check.
	pub health_check_id: String,
}

/// Struct to hold information about a worker that we need to perform health checks on.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerInfo {
	// Worker ID
	pub id: u64,
	// Worker networking information.
	pub network: Network,
}

pub async fn cockroach_connection() -> Result<PgPool, Error> {
	let db_url = dotenv!("COCKROACH_DB_URL");

	let pool = PgPool::connect(db_url).await?;
	Ok(pool)
}

pub async fn save_health_check_to_db(pool: &PgPool, health_check: HealthCheck) -> Result<(), SqlxError> {
	sqlx::query!(
		r#"
		INSERT INTO health_checks (path, port, method, tls_skip_verification, timeout, interval, grace_period, max_failures, type, headers, custom_health_check)
		VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
		ON CONFLICT (PATH) DO UPDATE
		SET port = excluded.port, method = excluded.method, tls_skip_verification = excluded.tls_skip_verification, 
		grace_period = excluded.grace_period, interval = excluded.interval, timeout = excluded.timeout, 
		max_failures = excluded.max_failures, type = excluded.type, headers = excluded.headers, custom_health_check = excluded.custom_health_check
		"#,
		health_check.path,
        health_check.port,
        health_check.method.as_deref(),
        health_check.tls_skip_verification,
        health_check.grace_period,
        health_check.interval,
        health_check.timeout,
        health_check.max_failures,
        health_check.r#type,
        health_check.headers,
        health_check.custom_health_check.map(|c| serde_json::to_value(c).unwrap()),
    )
	.execute(pool)
	.await?;

	Ok(())
}

fn deserialize_custom_health_check(custom_health_check: Option<&str>) -> Option<CustomCheckType> {
	custom_health_check.and_then(|custom_check| {
		serde_json::from_str(custom_check)
			.map_err(|err | {
				tracing::error!("Failed to deserialize custom health check: {}", err);
			})
			.ok()
	})
}
// This function creates a health check loop for a given HealthCheckConfig.
pub async fn schedule_health_checks(
	connection: Arc<Mutex<Connection>>,
	project_id: u64,
	worker: WorkerInfo,
	configs: Vec<HealthCheck>,
	tasks_map: Arc<Mutex<HashMap<String, HealthCheckTask>>>,
) -> Vec<String> {
	let mut tasks = vec![];

	for config in configs {
		let connection = Arc::clone(&connection);
		let worker_clone = worker.clone();
		let config_clone = config.clone();
		let region = dotenv!("REGION");
		let uuid = Uuid::new_v4();

		let key = format!(
			"pj:{}:wkr:{}:{}:{}",
			project_id, worker_clone.id, region, uuid
		);

		let key_clone = key.clone();

		let handle = tokio::spawn(async move {
			// Wait for the grace period before starting the health checks.
			sleep(Duration::from_millis(config_clone.grace_period)).await;

			let mut failure_count = 0;

			loop {
				// Run the health check and increment the failure count if it fails.
				let mut conn = connection.lock().await;
				if let Err(e) = run_http_health_check(&mut HealthCheckContext {
					connection: &mut *conn,
					key: key.clone(),
					project_id,
					worker: &worker_clone,
					config: &config_clone,
					region,
				})
				.await
				{
					tracing::error!("Error: {:?}", e);
					failure_count += 1;
				} else {
					failure_count = 0;
				}

				// Mark the worker as unavailable if the failure count exceeds the maximum.
				if failure_count > config_clone.max_failures {
					let _: () = conn.hset(&key.clone(), "available", false).await.unwrap();
				}

				// Wait for the interval before running the next health check.
				sleep(Duration::from_millis(config_clone.interval)).await;
			}
		});

		tasks_map.lock().await.insert(
			key_clone.clone(),
			HealthCheckTask {
				handle,
				health_check_id: key_clone.clone(),
			},
		);

		tasks.push(key_clone);
	}

	tasks
}

/// This helper function constructs the URL for a health check.
fn construct_url(worker: &WorkerInfo, config: &HealthCheck) -> String {
	format!(
		"http{}://{}:{}/{}",
		if config.r#type == HealthCheckType::HTTPS {
			"s"
		} else {
			""
		},
		worker.network.primary_ipv4,
		config.port,
		config.path.strip_prefix('/').unwrap_or(&config.path)
	)
}

/// This helper function constructs the headers for a health check.
fn construct_headers(config: &HealthCheck) -> HeaderMap {
	let mut headers = HeaderMap::new();
	if let Some(header_vec) = &config.headers {
		for header in header_vec {
			headers.insert(
				HeaderName::from_bytes(header.key.as_bytes()).unwrap(),
				HeaderValue::from_str(&header.value).unwrap(),
			);
		}
	}

	headers
}

pub struct HealthCheckContext<'a> {
	pub connection: &'a mut Connection,
	pub key: String,
	pub project_id: u64,
	pub worker: &'a WorkerInfo,
	pub config: &'a HealthCheck,
	pub region: &'a str,
}

async fn run_http_health_check(context: &mut HealthCheckContext<'_>) -> Result<(), Error> {
	let url = construct_url(context.worker, context.config);
	let headers = construct_headers(context.config);

	let client = Client::builder()
		.timeout(Duration::from_millis(context.config.timeout))
		.danger_accept_invalid_certs(context.config.tls_skip_verification.unwrap_or(false))
		.build()
		.unwrap();

	let response = match context.config.method {
		Some(HttpMethod::GET) => client.get(&url).headers(headers).send().await?,
		Some(HttpMethod::POST) => client.post(&url).headers(headers).send().await?,
		Some(HttpMethod::PUT) => client.put(&url).headers(headers).send().await?,
		Some(HttpMethod::DELETE) => client.delete(&url).headers(headers).send().await?,
		Some(HttpMethod::PATCH) => client.patch(&url).headers(headers).send().await?,
		_ => client.get(&url).headers(headers).send().await?,
	};

	context
		.connection
		.hset::<_, _, _, usize>(&context.key, "available", response.status().is_success())
		.await
		.unwrap();

	context
		.connection
		.hset::<_, _, _, usize>(
			&context.key,
			"last_health_check",
			chrono::Utc::now().to_rfc3339(),
		)
		.await
		.unwrap();
	
	/* 
	if let Some(custom_check) = &context.custom_check {
		if let Err(e) = // {
			todo!()
		}
	}
	*/

	Ok(())
}

pub async fn stop_health_check(
	tasks_map: Arc<Mutex<HashMap<String, HealthCheckTask>>>,
	health_check_id: String,
) {
	if let Some(task) = tasks_map.lock().await.remove(&health_check_id) {
		task.handle.abort(); // This will stop the health check.
	} else {
		println!("No health check found with ID {}", health_check_id);
	}
}


// Run the JSON value exists custom check
async fn run_json_value_exists_check(
	context: &mut HealthCheckContext<'_>,
	json_path: &str,
	expected_value: &serde_json::Value,
) -> Result<(), Error> {
	let url = construct_url(context.worker, context.config);

	let headers = construct_headers(context.config);

    let client = Client::builder()
        .timeout(Duration::from_millis(context.config.timeout))
        .danger_accept_invalid_certs(context.config.tls_skip_verification.unwrap_or(false))
        .build()
        .unwrap();

    let response = match context.config.method {
        Some(HttpMethod::GET) => client.get(&url).headers(headers).send().await?,
        Some(HttpMethod::POST) => client.post(&url).headers(headers).send().await?,
        Some(HttpMethod::PUT) => client.put(&url).headers(headers).send().await?,
        Some(HttpMethod::DELETE) => client.delete(&url).headers(headers).send().await?,
        Some(HttpMethod::PATCH) => client.patch(&url).headers(headers).send().await?,
        _ => client.get(&url).headers(headers).send().await?,
    };

    if response.status().is_success() {
        let body = response.text().await?;

        // Deserialize the response body as JSON
        let json_body: serde_json::Value = serde_json::from_str(&body)?;

        // Check if the JSON path exists and contains the expected value
        if json_path_exists(&json_body, json_path, expected_value) {
            context
                .connection
                .hset::<_, _, _, usize>(&context.key, "available", true)
                .await
                .unwrap();
        } else {
            context
                .connection
                .hset::<_, _, _, usize>(&context.key, "available", false)
                .await
                .unwrap();
        }
    } else {
        context
            .connection
            .hset::<_, _, _, usize>(&context.key, "available", false)
            .await
            .unwrap();
    }

    context
        .connection
        .hset::<_, _, _, usize>(
            &context.key,
            "last_health_check",
            chrono::Utc::now().to_rfc3339(),
        )
        .await
        .unwrap();

    // todo: include latency in the health check

    Ok(())
}

fn json_path_exists(json: &serde_json::Value, json_path: &str, expected_value: &serde_json::Value) -> bool {
	let segments: Vec<&str> = json_path.split('.').collect();

	let mut current_json = json;
	for segment in segments {
		if let Some(next_json) = current_json.get(segment) {
			current_json = next_json;
		} else {
			return false;
		}
	}

	current_json == expected_value
}