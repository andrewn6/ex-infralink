use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Enum for custom health check types
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CustomCheckType {
	JsonValueExists {
		json_path: String,
		expected_value: serde_json::Value,
	},
	ResponseContainersString(String),
	ResponseStatus(Vec<u16>),
}

// Enum for health check types
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomHealthCheck {
	pub check_type: CustomCheckType,
}
// Define a HealthCheckConfig struct for holding health check configuration.
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct HealthCheck {
	// <origin>/{path}
	pub path: String,
	// Exposed port for the end-user's application to run the health check for (e.g. 3000).
	pub port: u64,
	// The method for the health check, if applicable.
	pub method: Option<HttpMethod>,
	// Skip TLS verification for HTTPS health checks, if applicable.
	pub tls_skip_verification: Option<bool>,
	// Grace period for the health check - it's the time to wait for the application to start before running the health check. Measured in milliseconds.
	pub grace_period: u64,
	// Interval between health checks. Measured in milliseconds. Minimum value is 10000.
	pub interval: u64,
	// Timeout for the health check. Measured in milliseconds.
	pub timeout: u64,
	// Maximum number of failed health checks before the worker is considered unhealthy.
	pub max_failures: u64,
	// Type of health check.
	pub r#type: HealthCheckType,
	// Headers to include in the health check request.
	pub headers: Option<Vec<Header>>,
	// Custom health checks
	pub custom_health_check: Option<CustomHealthCheck>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum HttpMethod {
	GET,
	POST,
	PUT,
	DELETE,
	PATCH,
	OPTIONS,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum HealthCheckType {
	HTTPS,
	HTTP,
	TCP,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Header {
	pub key: String,
	pub value: String,
}