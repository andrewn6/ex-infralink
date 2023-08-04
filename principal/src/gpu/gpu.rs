use reqwest::{header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE}, Client, Response};
use serde_json::json;
use std::error::Error;

struct GpuManager {
    client: Client,
}

impl GpuManager {
    pub fn new(api_key: &str) -> GpuManager {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap());
        headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap());

        GpuManager {
            client: Client::builder()
                .default_headers(headers)
                .build()
                .unwrap(),
        }
    }


    pub async fn create_gpu_instance(&self, label: &str, hostname: &str,) -> Result<Response, Box<dyn Error + Send + Sync>> {
        let data = json!({
            "region": "ewr",
            "plan": "vcg",
            "label": label,
            "os_id": 215,
            "hostname": hostname,
            "tags": ["gpu", "instance"]
        });

        self.client.post("https://api.vultr.com/v2/instances")
            .json(&data)
            .send()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
    }

    pub async fn delete_gpu_instance(&self, instance_id: &str) -> Result<Response, Box<dyn Error + Send + Sync>> {
        let url = format!("https://api.vultr.com/v2/instances/{}", instance_id);

        self.client.delete(&url)
            .send()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
    }

    pub async fn list_gpu_instances(&self) -> Result<Vec<serde_json::Value>, Box<dyn Error + Send + Sync>> {
        let response = self.client 
            .get("https://api.vultr.com/v2/instances")
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;

        if let Some(instances) = data.get("instances").and_then(|v| v.as_array()) {
            let gpu_instances: Vec<_> = instances.iter()
                .filter(|instance| {
                    instance.get("plan")
                        .and_then(|v| v.as_str())
                        .map_or(false, |s| s == "vcg")
                })
                .cloned()
                .collect();
    
            Ok(gpu_instances)
        } else {
            Ok(Vec::new())
        }
    }
}