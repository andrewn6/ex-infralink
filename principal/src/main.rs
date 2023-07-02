pub mod providers;
pub mod shared_config;
use dotenv::dotenv;

pub mod rules;
pub mod manager;
pub mod db;

use rules::rules::create_manager_rules;

#[tokio::main]
async fn main() {
	// Load environment variables into runtime
	dotenv().unwrap();

	// Only uncomment if changes are made
	create_manager_rules();
}