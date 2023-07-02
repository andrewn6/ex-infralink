use reqwest::Client;
use std::collections::HashMap;
use std::str::FromStr;
use dotenv_codegen::dotenv;

// Vultr provider
use crate::providers::vultr::models::request::instance::InstanceBuilder;
use crate::providers::vultr::models::request::instance::InstanceType;
use crate::providers::vultr::models::request::instance::Instance;
use crate::rules::rules;
use crate::rules::rules::Rule;
use crate::shared_config::SharedConfig;
use crate::providers::vultr::models::request::region::Region;

const VULTR_API_KEY: &str = dotenv!("VULTR_API_KEY");

pub struct Manager {
    client: Client,
    rules: Vec<Rule>,
}

impl Manager {
    pub fn new(rules: Vec<Rule>) -> Self {
        let rules = rules::load_rules();
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

    pub async fn manage(&self, shared_config: SharedConfig) {
        loop {
           match self.get_instances().await {
            Ok(instances) => {
                let instance_count = self.count_instances(&instances);
                for rule in &self.rules {
                    match rule.provider.as_str() {
                        "vultr" => {
                            for region in &rule.regions {
                                let region = Region::from_str(region).unwrap();
                                let count = instance_count.get(region.to_string().as_str()).unwrap_or(&0);
                                match count {
                                   c if c < &&rule.instance_count => {
                                        println!("Need to start {} instances in region {}", rule.instance_count - c, region); 
                                        let instance = InstanceBuilder::new()
                                            .region(region.clone())
                                            .build(shared_config).await;
                                        instance.start(shared_config);
                                   },
                                   c if c > &rule.instance_count => {
                                        println!("Need to stop {} instances in region {}", c - rule.instance_count, region);
                                        for instance in instances.iter().filter(|i| i.region == region && i.provider == rule.provider) {
                                            instance.halt(shared_config).await;
                                        }
                                    },
                                    _ => (),
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => todo!(),
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