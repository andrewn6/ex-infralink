mod rollout {
    include!("../rollout.rs");
}

use bollard::Docker;
use bollard::container::{CreateContainerOptions, ListContainersOptions, Config, StartContainerOptions, RemoveContainerOptions};

use std::time::Duration;
use std::collections::HashMap;
use std::sync::Arc;

use tokio::time::sleep;
use tonic::{Request, Response, Status};
use rollout::{ExponentialStrategy, StartRolloutRequest, StartRolloutResponse, RolloutStrategy};

pub struct ExponentialServiceImpl {
    pub docker: Docker,
}

#[tonic::async_trait]
impl ExponentialStrategy for ExponentialServiceImpl {
    async fn start_rollout (
        &self,
        request: Request<StartRolloutRequest>,
    ) -> Result<Response<StartRolloutResponse>, Status> {
        let strategy = request.into_inner().strategy;

        match strategy {
            Some(strategy) => {
                match strategy.rollout {
                    Some(RolloutStrategy::Exponential(exp_strategy)) => {
                        match execute_exponential_rollout(self.docker.clone(), exp_strategy).await {
                            Ok(_) => Ok(Response::new(StartRolloutResponse {
                                message: "Exponential rollout started successfully".into()
                            })),
                            Err(e) => Err(Status::internal(e.to_string()))
                        }
                    }
                    _ => Err(Status::invalid_argument("Invalid or missing strategy")),
                }
            }
            _ => Err(Status::invalid_argument("Invalid or missing strategy")),
        }
    }
}

async fn execute_exponential_rollout(docker: Docker, strategy: ExponentialStrategy) -> Result<(), Box<dyn std::error::Error>> {
    let mut percentage = strategy.initial_percentage;

    for _ in 0..strategy.steps {
        println!("Deploying {}% of services", percentage);
        sleep(Duration::from_secs(strategy.interval_seconds as u64)).await;
        percentage *= 2;
    }

    Ok(())
}

async fn adjust_service_deployment(
    docker: Arc<Docker>,
    image_name: &str,
    container_base_name: &str,
    total_containers: i32,
    percentage: i32,
) -> Result<()> {
    let desired_container_count = (total_containers as f64 * (percentage as f64 / 100.0)).round() as i32;

    let mut filters = HashMap::new();
    filters.insert("name", vec![container_base_name]);

    let existing_containers = docker.list_containers(Some(ListContainersOptions {
        all: true,
        filters,
        ..Default::default()
    })).await?;

    let current_container_count = existing_containers.len() as i32;

    if desired_container_count > current_container_count {
        for _ in current_container_count..desired_container_count {
            let container_name = format!("{}-{}", container_base_name, current_container_count);

            docker.create_container(Some(CreateContainerOptions {
                name: container_name.clone(),
                platform: "linux/amd64",
            }), &bollard::container::Config {
                image: Some(image_name.to_string()),
                hostname: Some(container_name),
                domainname: None,
                user: None,
                attach_stdin: Some(false),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                exposed_ports: None,
                tty: Some(false),
                open_stdin: Some(false),
                stdin_once: Some(false),
                env: None,
                cmd: None,
                healthcheck: None,
                args_escaped: None,
                volumes: None,
                working_dir: None,
                entrypoint: None,
                network_disabled: Some(false),
                mac_address: None,
                on_build: None,
                labels: None,
                stop_signal: None,
                stop_timeout: None,
                shell: None,
                host_config: None,
                networking_config: None,
                ..Default::default()
            }).await?;            
            docker.start_container(&container_name, None).await.ok();
        }
    } else if desired_container_count < current_container_count {
        for container in existing_containers.into_iter().take((current_container_count - desired_container_count) as usize) {
            docker.remove_container(&container.names[0], Some(RemoveContainerOptions  {
                force: true,
                ..Default::default()
            })).await?;
        }
    }

    Ok(())
}