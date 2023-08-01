use std::error::Error;
use std::collections::HashMap;
use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION};
use serde::{Serialize, Deserialize};
use dotenv_codegen::dotenv;

const VULTR_API_KEY: &str = dotenv!("VULTR_API_KEY");
const HETZNER_API_KEY: &str = dotenv!("HETZNER_API_KEY");

#[derive(Debug, Clone)]
pub struct VolumeManager {
    client: Arc<reqwest::Client>,
}

/* Vultr Structs/Impls */
#[derive(Debug, Serialize, Deserialize)]
pub struct VultrVolumeConfig {
    region: String,
    size_gb: i32,
    label: String,
    block_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct VultrVolumeAttachmentConfig {
    instance_id: String,
    live: bool,
}

impl VultrVolumeAttachmentConfig {
    pub fn new(instance_id: &str, live: bool) -> Self {
        Self {
            instance_id: instance_id.to_string(),
            live,            
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct VultrVolumeDetachConfig {
    live: bool,
}

impl VultrVolumeDetachConfig {
    pub fn new(live: bool) -> Self {
        Self {
            live,
        }
    }
}

#[derive(Deserialize)]
struct VultrBlockResponse {
    id: String,
    date_created: String,
    size_gb: i32,
}

#[derive(Deserialize, Serialize)]
pub struct VultrBlock {
    #[serde(skip)]
    id: String,
    date_created: String,
    size_gb: i32,
}

#[derive(Deserialize)]
struct VultrBlocksResponse {
    blocks: Vec<VultrBlock>,
}

/* Hetzner Structs/Impls */
#[derive(Debug, Serialize, Deserialize)]
pub struct HetznerVolumeConfig {
    automount: bool,
    format: String,
    labels: HashMap<String, String>,
    location: String,
    name: String,
    size: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HetznerVolumeResizeConfig {
    size: i32,
}

impl HetznerVolumeResizeConfig {
    pub fn new(size: i32) -> Self {
        Self {
            size,
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct HetznerVolumeAttachmentConfig {
    automount: bool,
    server: i32,
}

impl HetznerVolumeAttachmentConfig {
    pub fn new(server: i32, automount: bool) -> Self {
        Self {
            server,
            automount
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct HetznerVolume {
    id: i32,
    name: String,
    size: i32,
    created: String,
}

#[derive(Deserialize)]
struct HetznerVolumeResponse {
    volume: HetznerVolume,
}

#[derive(Deserialize)]
struct HetznerVolumesResponse {
    volumes: Vec<HetznerVolume>,
}

#[derive(Deserialize, Serialize)]
enum Provider {
    Hetzner,
    Vultr,
}

#[derive(Serialize)]
pub struct Volume {
    id: String,
    provider: Provider,
}

impl VolumeManager {
    pub fn volume() -> Self {
        VolumeManager {
            client: Arc::new(reqwest::Client::new()),
        }
    }

    fn vultr_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", VULTR_API_KEY)).unwrap());
        headers
    }

    fn hetzner_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", HETZNER_API_KEY)).unwrap());
        headers
    }

    pub async fn get_all_volumes_hetzner(&self) -> Result<Vec<HetznerVolume>, Box<dyn Error>> {
        let response = self.client.get("https://api.hetzner.cloud/v1/volumes")
            .headers(self.hetzner_headers())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to get all volumes: {}", response.text().await?).into());
        }

        let volumes_response: HetznerVolumesResponse = response.json().await?;

        for volume in &volumes_response.volumes {
            println!("Volume ID: {}", volume.id);
            println!("Volume size: {} GB", volume.size);
            println!("Volume created at: {}", volume.created);
        }

        Ok(volumes_response.volumes)
    }

    pub async fn get_all_volumes_on_vultr(&self) -> Result<Vec<VultrBlock>, Box<dyn Error>> {
        let response = self.client.get("https://api.vultr.com/v2/blocks")
            .headers(self.vultr_headers())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to get volumes: {}", response.text().await?).into());
        }

        let volumes_response: VultrBlocksResponse = response.json().await?;

        for volume in &volumes_response.blocks {
            println!("Volume ID: {}", volume.id);
            println!("Volume size: {} GB", volume.size_gb);
            println!("Volume created at: {}", volume.date_created);
        }

        Ok(volumes_response.blocks)
    }

    pub async fn create_volume_on_vultr(&self, volume_config: VultrVolumeConfig)  -> Result<Volume, Box<dyn Error>> {
        let response = self.client.post("https://api.vultr.com/v2/blocks")
            .headers(self.vultr_headers())
            .json(&volume_config)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err("Failed to create volume".into());
        }

        let volume_response: VultrBlockResponse = response.json().await?;
        
        println!("Created a new volume on Vultr with ID: {}", volume_response.id);
        println!("Volume size: {} GB", volume_response.size_gb);
        println!("Volume created at: {}", volume_response.date_created);

        Ok(Volume {
            id: volume_response.id,
            provider: Provider::Vultr,
        })
    }

    pub async fn attach_volume_on_vultr(&self, volume_id: &str, config: VultrVolumeAttachmentConfig) -> Result<(), Box<dyn Error>> {
        let response = self.client.post(&format!("https://api.vultr.com/v2/blocks/{}/attach", volume_id))
            .headers(self.vultr_headers())
            .json(&config)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to attach volume {}: {}", volume_id, response.text().await?).into());
        }

        Ok(())
    }

    pub async fn resize_volume_on_vultr(&self, block_id: &str, new_label: &str, new_size_gb: i32) -> Result<(), Box<dyn Error>> {
        let resize_config = serde_json::json!({
            "label": new_label,
            "size_gb": new_size_gb,
        });

        let response = self.client.patch(&format!("https://api.vultr.com/v2/blocks/{}", block_id))
            .headers(self.vultr_headers())
            .json(&resize_config)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to resize volume {}: {}", block_id, response.text().await?).into());
        }

        println!("Resized volume with ID: {}", block_id);

        Ok(())
    }

    pub async fn detach_volume_on_vultr(&self, volume_id: &str, config: VultrVolumeDetachConfig) -> Result<(), Box<dyn Error>> {
        let response = self.client.post(&format!("https://api.vultr.com/v2/blocks/{}/detach", volume_id))
            .headers(self.vultr_headers())
            .json(&config)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to detach volume {}: {}", volume_id, response.text().await?).into());
        }

        Ok(())
    }

    pub async fn create_volume_on_hetzner(&self, volume_config: HetznerVolumeConfig) -> Result<Volume, Box<dyn Error>> {
        let response = self.client.post("https://api.hetzner.cloud/v1/volumes")
            .headers(self.hetzner_headers())
            .json(&volume_config)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to create volume: {}", response.text().await?).into());
        }

        let volume_response: HetznerVolumeResponse = response.json().await?;

        println!("Created a new volume with ID: {}", volume_response.volume.id);
        println!("Volume name: {}", volume_response.volume.name);
        println!("Volume size: {} GB", volume_response.volume.size);
        println!("Volume created at: {}", volume_response.volume.created);

        Ok(Volume {
            id: volume_response.volume.id.to_string(),
            provider: Provider::Hetzner
        })
    }

    pub async fn attach_volume_on_hetzner(&self, volume_id: &str, config: HetznerVolumeAttachmentConfig) -> Result<HetznerVolume, Box<dyn std::error::Error>> {
        let response = self.client.post(&format!("https://api.hetzner.cloud/v1/volumes/{}/actions/attach", volume_id))
            .headers(self.hetzner_headers())
            .json(&config)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to attach volume: {}: {}", volume_id, response.text().await?).into());
        }

        let volume_response: HetznerVolumeResponse = response.json().await?;

        Ok(volume_response.volume)
    }

    pub async fn resize_volume_on_hetzner(&self, volume_id: &str, config: HetznerVolumeResizeConfig) -> Result<(), Box<dyn Error>> {
        let response = self.client.post(&format!("https://api.hetzner.cloud/v1/volumes/{}/actions/resize", volume_id))
            .headers(self.hetzner_headers())
            .json(&config)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to resize volume {}: {}", volume_id, response.text().await?).into());
        }

        Ok(())
    }

    pub async fn detach_volume_hetzner(&self, volume_id: &str) -> Result<(), Box<dyn Error>> {
        let response = self.client.post(&format!("https://api.hetzner.cloud/v1/volumes/{}/actions/detach", volume_id))
            .headers(self.hetzner_headers())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to detach volume {}: {}", volume_id, response.text().await?).into());
        }

        Ok(())
    }
}