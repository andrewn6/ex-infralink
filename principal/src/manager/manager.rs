use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use sqlx::postgres::PgPool;
use dotenv_codegen::dotenv;
use crate::rules::rule::Rule;
use crate::shared_config::SharedConfig;

// Vultr provider
use crate::providers::vultr::models::request::instance::InstanceBuilder;
use crate::providers::vultr::models::request::instance::Instance;
use crate::providers::vultr::models::request::region::Region as VultrRegions;

// Hetzner
use crate::providers::hetzner::models::request::instance::InstanceBuilder as HetznerInstanceBuilder;
use crate::providers::hetzner::models::request::instance::Instance as HetznerInstance;
use crate::providers::hetzner::models::request::region::Region as HetznerRegions;
use crate::providers::vultr::models::request::region::NorthAmerica::NewJersey;

#[derive(Debug)]
pub enum ManagerError {
    DatabaseError(sqlx::Error),
    ReqwestError(reqwest::Error),
    ProviderError(String),
}

impl fmt::Display for ManagerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ManagerError::DatabaseError(e) => write!(f, "Database error: {}", e),
            ManagerError::ReqwestError(e) => write!(f, "Reqwest error: {}", e),
            ManagerError::ProviderError(e) => write!(f, "Provider error: {}", e),
        }
    }
}

impl Error for ManagerError {}

impl From<sqlx::Error> for ManagerError {
    fn from(e: sqlx::Error) -> Self {
        ManagerError::DatabaseError(e)
    }
}

impl From<reqwest::Error> for ManagerError {
    fn from(err: reqwest::Error) -> ManagerError {
        ManagerError::ReqwestError(err)
    }
}

#[derive(Debug)]
pub struct Manager {
    client: Client,
    rules: Vec<Rule>,
    pool: PgPool,
    vultr_key: String,
    hetzner_key: String,
}

#[derive(Debug)]
pub enum AnyInstance {
    Vultr(Box<Instance>),
    Hetzner(Box<HetznerInstance>),
}

impl Manager {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let database_url = dotenv!("COCKROACH_DB_URL");
        let pool = PgPoolOptions::new().connect(database_url).await?;
        let rules = Self::load_rules(&pool).await?;

        let vultr_key = dotenv!("VULTR_API_KEY").to_string();
        let hetzner_key = dotenv!("HETZNER_API_KEY").to_string();

        Ok(Self {
            client: Client::new(),
            rules,
            pool,
            vultr_key,
            hetzner_key,
        })
    }

    async fn load_rules(pool: &sqlx::Pool<sqlx::Postgres>) -> Result<Vec<Rule>, sqlx::Error> {
        let mut rules = vec![];
        let recs = sqlx::query_as::<_, (String, String, i32)>(
            r#"
            SELECT provider, region, instance_count
            FROM Providers
            "#,
        )
        .fetch_all(pool)
        .await?;
    
        

    for rec in recs {
        let rule = Rule {
            provider: rec.0,
            region: vec![rec.1],
            instance_count: rec.2,
        };
            rules.push(rule);
        }

        Ok(rules)
    }

    pub async fn get_instances(&self) -> Result<Vec<AnyInstance>, ManagerError> {
        let vultr_future = self.get_vultr_instances();
        let hetzner_future = self.get_hetzner_instances();

        let results = futures::try_join!(vultr_future, hetzner_future /*, oracle_future */); 

        match results {
            Ok((vultr_instances, hetzner_instances)) => {
                let mut instances: Vec<AnyInstance>  = Vec::new();

                for instance in vultr_instances {
                    instances.push(AnyInstance::Vultr(Box::new(instance)));
                }
                for instance in hetzner_instances {
                    instances.push(AnyInstance::Hetzner(Box::new(instance)));
                }

                Ok(instances)
            }
            Err(e) => Err(ManagerError::from(e))
        }
    }

    async fn get_vultr_instances(&self) -> Result<Vec<Instance>, reqwest::Error> {
        let resp = self.client.get("https://api.vultr.com/v2/instances")
            .bearer_auth(&self.vultr_key)
            .send()
            .await?
            .json::<Vec<Instance>>()
            .await?;
        
        Ok(resp)
    }

    /* 
    async fn get_oracle_instances(&self) -> Result<Vec<Instance>, reqwest::Error> {
        let resp = self.client.get("api.oracle.com/servers") // TODO: implement actual oracle route.
            .bearer_auth(&self.oracle_key)
            .send()
            .await?
            .json::<Vec<Instance>>()
            .await?;

        Ok(resp)
    }
    */
    
    async fn get_hetzner_instances(&self) -> Result<Vec<HetznerInstance>, reqwest::Error> {
        let resp = self.client.get("https://api.hetzner.cloud/v1/servers")
            .bearer_auth(&self.hetzner_key)
            .send()
            .await?
            .json::<Vec<HetznerInstance>>()
            .await?;

        Ok(resp)
    }

    pub async fn manage(&self, mut shared_config: SharedConfig) {
        loop {
            match self.get_instances().await {
                Ok(instances) => {
                    let instance_count = self.count_instances(&instances);
                    for rule in &self.rules {
                        match rule.provider.as_str() {
                            "vultr" => {
                                for _region_str in &rule.region {
                                    let region = VultrRegions::NorthAmerica(NewJersey);
                                    let count = instance_count.get(region.to_string().as_str()).unwrap_or(&0);
                                    match count {
                                        c if c < &rule.instance_count => {
                                            println!("Need to start {} instances in region {}", rule.instance_count - c, region);
                                            let instance = InstanceBuilder::new()
                                                .region(region.clone())
                                                .build(&mut shared_config).await;
                                            instance.start(&mut shared_config).await;                            
                                        },
                                        c if c > &rule.instance_count => {
                                            println!("Need to stop {} instances in region {}", c - rule.instance_count, region);
                                            for any_instance in instances.iter().filter(|i| {
                                                if let AnyInstance::Vultr(instance) = i {
                                                    instance.region == region && instance.provider == rule.provider
                                                } else {
                                                    false
                                                }
                                            }) {
                                                if let AnyInstance::Vultr(instance) = any_instance {
                                                    instance.halt(&mut shared_config).await;
                                                }
                                            }
                                        }
                                        _ => (),
                                    }
                                }
                            }
                            "hetzner" => {
                                for _region_str in &rule.region {
                                    let region = HetznerRegions::Helsinki;
                                    let count = instance_count.get(region.to_string().as_str()).unwrap_or(&0);
                                    match count {
                                        c if c < &rule.instance_count => {
                                            println!("Need to start {} instances in region {:?}", rule.instance_count - c, region);
                                            let instance = HetznerInstanceBuilder::new()
                                                .region(region.clone())
                                                .build(&mut shared_config).await;
                                            instance.start(&mut shared_config).await;
                                        }
                                       c if c > &rule.instance_count => {
                                            println!("Need to stop {} instances in region {:?}", rule.instance_count - c, region);
                                            for any_instance in instances.iter().filter(|i| {
                                                if let AnyInstance::Hetzner(instance) = i {
                                                    instance.region == region && instance.provider == rule.provider
                                                } else {
                                                    false
                                                }
                                            }) {
                                                if let AnyInstance::Vultr(instance) = any_instance {
                                                    instance.halt(&mut shared_config).await;
                                                }
                                            }
                                       }
                                        _ => (),
                                    }
                                }
                            }
                            _ => {
                                println!("Unsupported provider: {}", rule.provider);
                            }
                        }
                    }
                }
                Err(_) => todo!()
            }
        }
    }

    fn count_instances(&self, instances: &Vec<AnyInstance>) -> HashMap<String, i32> {
        let mut instance_count: HashMap<String, i32> = HashMap::new();

        for any_instance in instances {
            let region = match any_instance {
                AnyInstance::Vultr(instance) => instance.region.to_string(),
                AnyInstance::Hetzner(instance) => instance.region.to_string()
            };

            let count = instance_count.entry(region).or_insert(0);
            *count += 1;
        }

        instance_count
    }
}