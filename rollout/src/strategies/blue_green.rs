mod rollout {
    include!("../rollout.rs");
}

use std::net::SocketAddr;
use std::collections::HashMap;

use surf::StatusCode;
use tonic::{Request, Response, Status};

use bollard::Docker;
use bollard::models::{ContainerCreateOptions, HostConfig, ContainerListOptions};

use rollout::{RolloutStrategy, BlueGreenStrategy, StrategyType};
use rollout::rollout_strategy::Rollout;
use surf::{Client, http::Url};

use self::rollout::rollout_strategy;

const CONTAINER_PREFIX: &str = "my_app_version_";
const HEALTH_CHECK_PATH: &str = "/health";
const OLD_APP_ADDR: &str = "http://127.0.0.1:7000"; // Replace with the actual address of the old version
const NEW_APP_ADDR: &str = "http://127.0.0.1:7001";


pub struct LoadBalancer {
    current_version: String,
    old_version: String,
    new_version: String,
}

pub struct BlueGreenRollout {
    docker: Docker,
    load_balancer: LoadBalancer,
    containers: HashMap<String, String>,
}

// Very basic this will need to be improved and expanded eventually.. works for now though.
impl LoadBalancer {
    pub fn new(old_version: String, new_version: String) -> Self {
        Self {
            current_version: old_version.clone(),
            old_version,
            new_version,
        }
    }

    pub fn get_next_version(&mut self) -> String {
        if self.current_version == self.old_version {
            self.current_version = self.new_version.clone();
            &self.new_version
        } else {
            self.current_version = self.old_version.clone();
            &self.old_version
        }
    }
}

#[tonic::async_trait]
impl rollout::BlueGreen for BlueGreenService {
    async fn deploy(
        &self,
        request: Request<BlueGreenRequest>,
    ) -> Result<Response<DeployResponse>, Status> {
        let deploy_request = request.into_inner();
        if let Some(strategy) = deploy_request.strategy {
            if strategy.r#type == StrategyType::BlueGreen as i32 {
                if let Some(rollout) = strategy.rollout {
                    match rollout {
                        Rollout::BlueGreen(blue_green) => {
                            let new_image = blue_green.blue_green_field;
                            let version = if new_image { "new" } else { "old" };
                            let mut rollout_strategy = rollout::BlueGreenRollout::new("old".to_string(), "new".to_string()).await?;
                            rollout_strategy.reverse_proxy_to_version(version).await?;
                            return Ok(Response::new(rollout::DeployResponse { success: true }));
                        },
                        _ => return Err(Status::invalid_argument("Invalid blue-green roll out strategy")),
                    }
                } else {
                    return Err(Status::invalid_argument("Missing blue-green roll out strategy"));
                }
            } else {
                return Err(Status::invalid_argument("Invalid roll out strategy"));
            }
        } else {
            return Err(Status::invalid_argument("Missing roll out strategy"));
        }
    }
}

impl BlueGreenRollout {
    pub async fn new() -> Result<Self, Status> {
        let docker = Docker::connect_with_local_defaults().map_err(|_| Status::unavailable("Failed to connect to Docker"))?;
        Ok(Self {
             docker,
             load_balancer: LoadBalancer::new(old_version, new_version),
             containers: HashMap::new(),
        })
    }

    async fn reverse_proxy_to_version(&self, version: &str) -> Result<(), Status> {
        let version_addr = self.get_version_addr(version);

        if !self.is_container_running(version).await? {
            self.create_start_container(version, &version_addr).await?;
        }

        let version_url = Url::parse(&version_addr).expect("Invalid version URL");

        let request = surf::Request::builder
            .method("GET")
            .uri(version_url)
            .body(surf::Body::empty())
            .expect("failed to build http request");

        let mut response = surf::client().send(request).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Status::unavailable("Failed to proxy to new version"))
        }
    }

    async fn is_container_running(&self, version: &str) -> Result<bool, Status> {
        let container_name = format!("{}{}", CONTAINER_PREFIX, version);
        let options = ContainerListOptions::builder().build();
        let containers = self.docker.list_containers(Some(options)).await.map_err(|_| Status::unavailable("Failed to list containers"))?;

        for container in containers {
            if let Some(names) = container.names {
                for name in names {
                    if name == format!("/{}", container_name) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    async fn create_and_start_container(&self, version: &str, versio_addr: &str) -> Result<(), Status> {
        let container_name = format!("{}{}", CONTAINER_PREFIX, version);
        let create_options = ContainerCreateOptions::builder()
            .name(&container_name)
            .image(version)
            .host_config(HostConfig::builder().publish_all_ports(true).build())
            .build();

        self.docker.create_container(Some(create_options)).await.map_err(|_| Status::unavailable("Failed to create container"))?;
        self.docker.start_container(&container_name, None::<String>).await.map_err(|_| Status::unavailable("Failed to start container"))?;

        Ok(())
    }

    async fn cleanup_containers(&mut self, keep_version: &str) -> Result<(), Status> {
        let options = ContainerListOptions::builder().all(true).build();
        let containers = self.docker.list_containers(Some(options)).await.map_err(|_| Status::unavailable("Failed to list containers"))?;

        for container in containers {
            if let Some(names) = containerr.names {
                for name in names {
                    if let Some(container_id) = self.containers.remove(&name[1..]) {
                        let options = bollard::container::RemoveContainerOptions {
                            force: true,
                            ..Default::default()
                        };
                        if let Err(_) = self.docker.remove_container(&container_id, Some(options)).await {
                            return Err(Status::unavailable("Failed to remove container"));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn rollback_to_version(&mut self, version: &str) -> Result<(), Status> {
        self.cleanup_containers(version).await?;
        self.load_balancer = LoadBalancer::new(version.to_string(), self.load_balancer.new_version.clone());

        Ok(())
    }

    fn get_version_addr(&self, version: &str) -> String {
        match version {
            "old" => OLD_APP_ADDR.to_string(),
            "new" => NEW_APP_ADDR.to_string(),
            _ => panic!("Invalid version"),
        }
    }
}