use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::collections::HashMap;
use dotenv_codegen::dotenv;

const VULTR_API_KEY: &str = dotenv!("VULTR_API_KEY");

#[derive(Debug, Deserialize, Serialize)]
pub struct Rule {
    pub provider: String,
    pub regions: Vec<String>,
    pub instance_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Volume {
	pub id: u64,
	pub used: u64,
	pub total: u64,
	pub r#type: VolumeType,
	pub tier: VolumeTier,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum VolumeTier {
	HighPerformance,
	UltraHighPerformance,
	ExtremePerformance,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum VolumeType {
	NVME,
	SATA,
	HDD,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Instance {
    pub provider: String,
    pub region: String,
    pub vcpu: u64,
    pub memory: u64,
    pub boot_volume: Volume,
}

pub struct InstanceManager {
    client: Client,
    rules: Vec<Rule>,
}

impl InstanceManager {
    pub fn new(rules: Vec<Rule>) -> Self {
        Self {
            client: Client::new(),
            rules,
        }
    }

    pub async fn get_instances(&self) -> Result<Vec<Instance>, reqwest::Error> {
        /* TODO:, make a function that gets all the pre-warmed instances from all cloud platforms. */
        let resp = self.client.get("https://api.vultr.com/v2/instances")
            .bearer_auth(VULTR_API_KEY)
            .send()
            .await?
            .json::<Vec<Instance>>()
            .await?;

        Ok(resp)
    }

    pub async fn manage(&self) {
        loop {
            match self.get_instances().await {
                Ok(instances) => {
                    let instance_count = self.count_instances(&instances);
                    for rule in &self.rules {
                        if rule.provider != "vultr" {
                           continue; 
                        }
                        for region in &rule.regions {
                            let count = instance_count.get(region).unwrap_or(&0);
                            match count {
                                c if c < &rule.instance_count => {
                                    println!("Need to start {} instances in region {}", rule.instance_count - c, region);
                                    // TODO: Start the warmed instance
                                },
                                c if c > &rule.instance_count => {
                                    println!("Need to stop {} instances in region {}", c - rule.instance_count, region);
                                    // TODO: stop instance
                                },
                                _ => (), 
                            }
                        }
                    }
                },
                Err (e) => {
                    println!("Error getting instances: {:?}", e);
                },
            }
        }
    }

    fn count_instances(&self, instances: &[Instance]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for instance in instances {
            *counts.entry(instance.region.clone()).or_insert(0) += 1;
        }
        counts
    }
}