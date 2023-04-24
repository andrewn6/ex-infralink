pub mod compute;
pub mod memory;
pub mod network;
#[derive(Debug, Clone)]
pub struct NetworkMetadata {
    pub network: Option<u64>,
}

impl NetworkMetadata {
    pub fn new() -> Self {
        Self { network: None }
    }
}
