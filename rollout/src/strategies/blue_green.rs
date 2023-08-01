mod rollout {
    include!("../rollout.rs");
}

use std::net::SocketAddr;
use surf::StatusCode;
use tonic::{Request, Response, Status};

use bollard::Docker;
use bollard::models::{ContainerCreateOptions, HostConfig, ContainerListOptions};

use rollout::{RolloutStrategy, StrategyType};
use rollout::rollout_strategy::Rollout;
use surf::{Client, http::Url};

const CONTAINER_PREFIX: &str = "my_app_version_";
const HEALTH_CHECK_PATH: &str = "/health";
const OLD_APP_ADDR: &str = "http://127.0.0.1:7000"; // Replace with the actual address of the old version
const NEW_APP_ADDR: &str = "http://127.0.0.1:7001";

#[derive(Default)]
pub struct BlueGreenRollout {
    docker: Docker,
}


impl BlueGreenRollout {
    pub async fn new() -> Result<Self, Status> {
        let docker = Docker::connect_with_local_defaults().map_err(|_| Status::unavailable("Failed to connect to Docker"))?;
        Ok(Self { docker })
    }
}

#[tonic::async_trait]
impl BlueGreen for BlueGreenService {
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
                            self.reverse_proxy_to_version(version).await?;
                            return Ok(Response::new(DeployResponse { success: true }));
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

    fn get_version_addr(&self, version: &str) -> String {
        match version {
            "old" => OLD_APP_ADDR.to_string(),
            "new" => NEW_APP_ADDR.to_string(),
            _ => panic!("Invalid version"),
        }
    }
}