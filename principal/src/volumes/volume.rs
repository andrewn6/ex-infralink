use std::error::Error;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize)]
pub struct VolumeConfig {
    region: String,
    size_gb: i32,
    label: String,
    block_type: String,
}
pub struct VolumeManager {
    config: VolumeManagerConfig,
    client: reqwest::Client,
}

#[derive(Deserialize)]
struct VultrBlockResponse {
    id: String,
    date_created: String,
    size_gb: i32,
}

#[derive(Debug)]
pub struct VolumeManagerConfig {
    hetzner_api_key: String,
    vultr_api_key: String,
}

enum Provider {
    Hetzner,
    Vultr,
}

pub struct Volume {
    id: String,
    provider: Provider,
}

impl VolumeManager {
    pub fn new(config: VolumeManagerConfig) -> VolumeManager {
        VolumeManager {
            config, 
            client: reqwest::Client::new(),
        }
    }

    fn vultr_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", self.config.vultr_api_key)).unwrap());
        headers
    }

    pub async fn create_volume_on_vultr(&self, volume_config: VolumeConfig)  -> Result<Volume, Box<dyn Error>> {
        let response = self.client.post("https://api.vultr.com/v2/blocks")
            .headers(self.vultr_headers())
            .json(&volume_config)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err("Failed to create volume".into());
        }

        let volume_response: VultrBlockResponse = response.json().await?;
        
        println!("Created a new volume with ID: {}", volume_response.id);
        println!("Volume size: {} GB", volume_response.size_gb);
        println!("Volume created at: {}", volume_response.date_created);

        Ok(Volume {
            id: volume_response.id,
            provider: Provider::Vultr,
        })
    }

    pub async fn create_volume_on_hetzner(&self, volume_config: VolumeConfig) -> Result<Volume, Box<dyn Error>> {
        todo!()
    }
}