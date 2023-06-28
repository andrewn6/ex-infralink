use serde_yaml::from_reader;
use std::fs::File;
use std::io::BufReader;
use serde::{Deserialize, Serialize};

/* Used for pre-warmed instance rules */
#[derive(Debug, Deserialize, Serialize)]
pub struct Rule {
    pub provider: String,
    pub regions: Vec<String>,
    pub instance_count: usize,
}

pub fn load_rules() -> Vec<Rule> {
    let file = File::open("../../config/principal/prewarm_rules.yaml").unwrap();
    let reader = BufReader::new(file);
    let rules: Vec<Rule> = from_reader(reader).unwrap();
    rules
}