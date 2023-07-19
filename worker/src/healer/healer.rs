mod healer {
    include!("../healer.rs");
}

use std::time::{Duration, SystemTime};
use std::sync::Arc;

use futures_util::StreamExt;
use tonic::{Request, Response, Status};
use tokio::time;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};

use bollard::container::{CreateContainerOptions, RemoveContainerOptions, ListContainersOptions};
use bollard::Docker;

use crate::proto_healer::healer_server::Healer;
use crate::proto_healer::{
    StartHealingRequest,
    StartHealingResponse,
    StopHealingRequest,
    StopHealingResponse,
    HealSelectiveRequest,
    HealSelectiveResponse,
    GetHealingReportRequest,
    GetHealingReportResponse,
    HealingReport,
    UpdateRequest,
    UpdateResponse,
};

#[derive(Clone)]
pub struct MyHealer {
    pub docker: Docker,
    pub create_options: CreateContainerOptions<String>,
    pub container_config: bollard::container::Config<String>,
    pub healing: Arc<Mutex<bool>>,
    pub healing_report: Arc<Mutex<Vec<HealingReport>>>,
}

impl MyHealer {
    pub fn new(docker: Docker, create_options: CreateContainerOptions<String>, container_config: bollard::container::Config<String>) -> Self {
        MyHealer {
            docker,
            create_options,
            container_config,
            healing: Arc::new(Mutex::new(false)),
            healing_report: Arc::new(Mutex::new(Vec::new())),
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
                                    } else {
                                        let system_time = SystemTime::now();
                                        let datetime: DateTime<Utc> = system_time.into(); 
                                        let timestamp_str = datetime.to_rfc3339();

                                        let healing_report = &mut *self.healing_report.lock().await;
                                        healing_report.push(HealingReport {
                                            container_id: container_id.clone(),
                                            timestamp: timestamp_str,
                                            event: "Container was healed".to_string(),
                                        });
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
    
    pub async fn heal_containers(&self, containers_id: Vec<String>) {
        for container_id in containers_id { 
            let mut stats_stream = self.docker.stats(&container_id, None);
            if let Some(Ok(stat)) = stats_stream.next().await {
                if stat.read.is_empty() {
                    self.docker.remove_container(&container_id, Some(RemoveContainerOptions {
                        force: true,
                        ..Default::default()
                    })).await.unwrap();

                    if let Err(e) = self.docker.create_container(Some(self.create_options.clone()), self.container_config.clone()).await {
                        eprintln!("Error creating container: {}", e);                   
                    } else {
                        let healing_report = &mut *self.healing_report.lock().await;

                        let system_time = SystemTime::now();
                        let datetime: DateTime<Utc> = system_time.into(); // Converts SystemTime to DateTime
                        let timestamp_str = datetime.to_rfc3339();

                        healing_report.push(HealingReport {
                            container_id: container_id.clone(),
                            timestamp: timestamp_str,
                            event: "Container was healed".to_string(),
                        });
                    }
                }
            }
        }
    }

    pub async fn perform_update(&self, containers_id: Vec<String>) {
        for container_id in containers_id {
            self.docker.remove_container(&container_id, Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            })).await.unwrap();

            if let Err(e) = self.docker.create_container(Some(self.create_options.clone()), self.container_config.clone()).await {
                eprintln!("Error creating container: {}", e)
            } else {
                let healing_report = &mut *self.healing_report.lock().await;

                let system_time = SystemTime::now();
                let datetime: DateTime<Utc> = system_time.into(); 
                let timestamp_str = datetime.to_rfc3339();

                healing_report.push(HealingReport {
                    container_id: container_id.clone(),
                    timestamp: timestamp_str,
                    event: "Container was healed".to_string(),
                });
            }

            time::sleep(Duration::from_secs(10)).await;
        }
    }
}

#[tonic::async_trait]
impl crate::proto_healer::healer_server::Healer for MyHealer {
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

    async fn heal_selective(
        &self,
        request: Request<HealSelectiveRequest>,
    ) -> Result<Response<HealSelectiveResponse>, Status> {
        let req = request.into_inner();
        let healer = self.clone();
        tokio::spawn(async move {
            healer.heal_containers(req.container_ids).await;
        });
        Ok(Response::new(HealSelectiveResponse {
            message: "Healing process started for selected containers".to_string(),
        }))
    }

    async fn get_healing_report(
        &self,
        _request: Request<GetHealingReportRequest>,
    ) -> Result<Response<GetHealingReportResponse>, Status> {
        let report = self.healing_report.lock().await.clone();
        Ok(Response::new(GetHealingReportResponse {
            healing_events: report,
        }))
    }

    async fn perform_rolling_update(
        &self,
        request: Request<UpdateRequest>,
    ) -> Result<Response<UpdateResponse>, Status> {
        let healer = self.clone();

        let containers_ids = request.into_inner().container_ids;
        tokio::spawn(async move {
            healer.perform_update(containers_ids).await;
        });

        Ok(Response::new(UpdateResponse {
            message: "Rolling update started".to_string()
        }))
    } 
}