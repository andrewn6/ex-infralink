use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Rule {
    pub provider: String,
    pub region: Vec<String>,
    pub instance_count: i32,
}