/* Used for pre-warmed instance rules */
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Rule {
    pub provider: String,
    pub regions: Vec<String>,
    pub instance_count: usize,
}