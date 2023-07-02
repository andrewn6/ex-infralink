use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use dotenv_codegen::dotenv;

#[derive(Debug, Clone)]
pub struct Rule {
    pub provider: String,
    pub regions: Vec<String>,
    pub instance_count: i32,
}

pub async fn create_manager_rules() -> Result<(), Box<dyn std::error::Error>> {
    let database_url  = dotenv!("COCKROACH_DB_URL");
    let pool = PgPoolOptions::new().connect(database_url).await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS Providers (
            id SERIAL PRIMARY KEY,
            provider TEXT NOT NULL,
            region TEXT NOT NULL,
            instance_count INT NOT NULL,
            UNIQUE(provider, region)
        )
        "#,
    )    
    .execute(&pool)
    .await?;

    let providers = vec![
        ("vultr", "us-west", 1),
        ("vultr", "us-east", 1),
        ("host_hatch", "eu-west", 1),
    ];

    for (provider, region, instance_count) in providers {
        let result = sqlx::query(
            r#"
            INSERT INTO Providers (provider, region, instance_count)
            VALUES ($1, $2, $3)
            ON CONFLICT (provider, region) DO NOTHING
            "#,
        )
        .bind(provider)
        .bind(region)
        .bind(instance_count)
        .execute(&pool)
        .await;

        match result {
            Ok(_) => println!("Data inserted or already exists."),
            Err(err) => {
                println!("Failed to insert data: {}", err);
            }
        }
    }

    Ok(())
}