use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use sqlx::postgres::PgPool;
use dotenv_codegen::dotenv;

// Vultr provider
use crate::providers::vultr::models::request::instance::InstanceBuilder;
use crate::providers::vultr::models::request::instance::Instance;
use crate::rules::rules::Rule;
use crate::shared_config::SharedConfig;
use crate::providers::vultr::models::request::region::Region;

const VULTR_API_KEY: &str = dotenv!("VULTR_API_KEY");

pub struct Manager {
    client: Client,
    rules: Vec<Rule>,
    pool: PgPool,
}

impl Manager {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let database_url = dotenv!("COCKROACH_DB_URL");
        let pool = PgPoolOptions::new().connect(database_url).await?;
        let rules = Self::load_rules(&pool).await?;
        Ok(Self {
            client: Client::new(),
            rules,
            pool,
        })
    }

    async fn load_rules(pool: &sqlx::Pool<sqlx::Postgres>) -> Result<Vec<Rule>, sqlx::Error> {
        let rules = vec![];
        let recs = sqlx::query(
            r#"
            SELECT provider, region, instance_count
            FROM Providers
            "#,
        )
        .fetch_all(pool)
        .await?;

    for rec in recs {
        let rule = Rule {
            provider: rec.provider,
            regions: vec![rec.region],
            instance_count: rec.instance_count,
        };
            rules.push(rule);
        }

        Ok(rules)
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
                                for region_str in &rule.regions {
                                    let region = Region::from(region_str.as_str());
                                    let count = instance_count.get(region.to_string().as_str()).unwrap_or(&0);
                                    match count {
                                        c if c < &&rule.instance_count => {
                                            println!("Need to start instances in region {}", rule.instance_count - c, region);
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
                                        // other providers go here, eg hosthatch, etc
                                        _ => (),
                                    }
                                }
                            }
                        }
                    }
                }
                Err(_) => todo!()
            }
        }
    }

    /* 
    fn count_instances(&self, instances: &[Instance]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for instance in instances {
            *counts.entry(instance.region.clone()).or_insert(0) += 1;
        }
        counts
    }
    */
}