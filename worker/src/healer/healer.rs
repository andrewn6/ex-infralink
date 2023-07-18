mod healer {
    include!("../healer.rs");
}

use std::time::Duration;
use std::sync::Arc;

use futures_util::StreamExt;
use tonic::{Request, Response, Status};
use tokio::time;
use tokio::sync::Mutex;

use bollard::container::{CreateContainerOptions, RemoveContainerOptions, ListContainersOptions};
use bollard::Docker;

use healer::healer_server::Healer;
use healer::StartHealingRequest;
use healer::StartHealingResponse;
use healer::StopHealingRequest;
use healer::StopHealingResponse;

#[derive(Clone)]
pub struct MyHealer {
    docker: Docker,
    create_options: CreateContainerOptions<String>,
    container_config: bollard::container::Config<String>,
    healing: Arc<Mutex<bool>>,
}

impl MyHealer {
    pub fn new(docker: Docker, create_options: CreateContainerOptions<String>, container_config: bollard::container::Config<String>) -> Self {
        MyHealer {
            docker,
            create_options,
            container_config,
            healing: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn start(self) {
        let mut healing = self.healing.lock().await;
        *healing = true;

        let mut backoff = Duration::from_secs(1);
        while *self.healing.lock().await {
            match self.docker.list_containers(Some(ListContainersOptions::<String>{
                all: true,
                ..Default::default()
            })).await {
                Ok(containers) => {
                    for container in containers {
                        if let Some(container_id) = &container.id {
                            let mut stats_stream = self.docker.stats(container_id, None);
            
                            if let Some(Ok(stat)) = stats_stream.next().await {
                                if stat.read.is_empty() {
                                    self.docker.remove_container(container_id, Some(RemoveContainerOptions {
                                        force: true,
                                        ..Default::default()
                                    })).await.unwrap();
    
                                    if let Err(e) = self.docker.create_container(Some(self.create_options.clone()), self.container_config.clone()).await {
                                        eprintln!("Error creating container: {}", e);
                                    }
                                }
                            }
                        }
                    }
    
                    backoff = Duration::from_secs(1);
                },
                Err(e) => {
                    eprintln!("Error listing containers: {}", e);
                    backoff = std::cmp::min(backoff * 2, Duration::from_secs(60));
                }
            }
    
            time::sleep(backoff).await;
        }
    }
    

    pub async fn stop(self) {
        let mut healing = self.healing.lock().await;
        *healing = false;
    }
}

#[tonic::async_trait]
impl Healer for MyHealer {
    async fn start_healing(
        &self,
        _request: Request<StartHealingRequest>,
    ) -> Result<Response<StartHealingResponse>, Status> {
        let healer = self.clone();

        tokio::spawn(async move {
            healer.start().await;
        });

        Ok(Response::new(StartHealingResponse {
            message: "Healing process started".to_string()
        }))
    }

    async fn stop_healing(
        &self,
        _request: Request<StopHealingRequest>,
    ) -> Result<Response<StopHealingResponse>, Status> {
        let healer = self.clone();

        tokio::spawn(async move {
            healer.stop().await;
        });

        Ok(Response::new(StopHealingResponse {
            message: "Healing process stopped".to_string()
        }))
    }

    async fn heal(
        &self,
        _request: Request<healer::HealRequest>,
    ) -> Result<Response<healer::HealResponse>, Status> {
        Ok(Response::new(healer::HealResponse {
            message: "Healing process started".to_string()
        }))
    }
}