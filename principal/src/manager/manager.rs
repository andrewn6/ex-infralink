use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use sqlx::Pool;
use sqlx::postgres::PgPool;
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
        let mut rules = vec![];
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

    
    fn count_instances(&self, instances: &[Instance]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for instance in instances {
            *counts.entry(instance.region.clone()).or_insert(0) += 1;
        }
        counts
    }
}