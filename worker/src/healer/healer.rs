mod healer {
    include!("../healer.rs");
}

use std::time::Duration;

use futures_util::StreamExt;
use tonic::{Request, Response, Status};
use tonic::transport::Server;
use tokio::time;

use bollard::container::{CreateContainerOptions, RemoveContainerOptions, ListContainersOptions};
use bollard::Docker;

use healer::healer_server::Healer;
use healer::StartHealingRequest;
use healer::StartHealingResponse;
use healer::StopHealingRequest;
use healer::StopHealingResponse;

pub struct MyHealer {
    docker: Docker,
    create_options: CreateContainerOptions<String>,
    container_config: bollard::container::Config<String>,
    healing: bool,
}

impl MyHealer {
    pub fn new(docker: Docker, create_options: CreateContainerOptions<String>, container_config: bollard::container::Config<String>) -> Self {
        MyHealer {
            docker,
            create_options,
            container_config,
            healing: false,
        }
    }

    pub async fn start(&mut self) {
        self.healing = true;

        while self.healing {
            let containers = self.docker.list_containers(Some(ListContainersOptions::<String>{
                all: true,
                ..Default::default()
            })).await.unwrap();

            for container in containers {
                if let Some(container_id) = &container.id {
                    let mut stats_stream = self.docker.stats(container_id, None);
            
                    if let Some(Ok(stat)) = stats_stream.next().await {
                        if stat.read.is_empty() {
                            self.docker.remove_container(container_id, Some(RemoveContainerOptions {
                                force: true,
                                ..Default::default()
                            })).await.unwrap();
            
                            self.docker.create_container(Some(self.create_options.clone()), self.container_config.clone()).await.unwrap();
                        }
                    }
                }
            }           

            time::sleep(Duration::from_secs(60)).await;
        }
    }

    pub fn stop(&mut self) {
        self.healing = false;
    }
}

#[tonic::async_trait]
impl Healer for MyHealer {
    async fn start_healing(
        &self,
        _request: Request<StartHealingRequest>,
    ) -> Result<Response<StartHealingResponse>, Status> {
        self.start().await;
        Ok(Response::new(StartHealingResponse {
            message: "Healing process started".to_string()
        }))
    }

    async fn stop_healing(
        &self,
        _request: Request<StopHealingRequest>,
    ) -> Result<Response<StopHealingResponse>, Status> {
        self.stop();
        Ok(Response::new(StopHealingResponse {
            message: "Healing process stopped".to_string()
        }))
    }

}