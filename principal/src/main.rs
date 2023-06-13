pub mod providers;
pub mod shared_config;

use dotenv::dotenv;

pub mod db;
pub mod scale;

#[tokio::main]
async fn main() {
	// Load environment variables into runtime
	dotenv().unwrap();
}
