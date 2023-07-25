#[derive(Debug, Clone)]
pub struct Rule {
    pub provider: String,
    pub regions: Vec<String>,
    pub instance_count: i32,
}