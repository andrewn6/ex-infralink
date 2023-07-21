mod healer {
    include!("../healer.rs");
}

use prometheus::{Opts, Registry, Counter, Encoder, TextEncoder};
use std::time::{Duration, SystemTime};
use std::sync::Arc;
use std::collections::HashMap;

use futures_util::StreamExt;
use tonic::{Request, Response, Status};
use tokio::time;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};

use bollard::container::{CreateContainerOptions, RemoveContainerOptions, ListContainersOptions};
use bollard::Docker;

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
    UpdateResponse, MetricsResponse,
    Empty,
};

#[derive(Clone)]
pub struct MyHealer {
    pub docker: Docker,
    pub create_options: CreateContainerOptions<String>,
    pub container_config: bollard::container::Config<String>,
    pub healing: Arc<Mutex<bool>>,
    pub healing_report: Arc<Mutex<Vec<HealingReport>>>,
    pub container_healed_count: Counter,
    pub heal_attempts: Arc<Mutex<HashMap<String, u32>>>,
}

impl MyHealer {
    pub fn new(docker: Docker, create_options: CreateContainerOptions<String>, container_config: bollard::container::Config<String>, registry: &Registry) -> Result<Self, Box<dyn std::error::Error>> {
        let counter_opts = Opts::new("container_healed_count", "Number of containers healed");
        let container_healed_count = Counter::with_opts(counter_opts)?;
        registry.register(Box::new(container_healed_count.clone()))?;

        Ok(MyHealer {
            docker,
            create_options,
            container_config,
            healing: Arc::new(Mutex::new(false)),
            healing_report: Arc::new(Mutex::new(Vec::new())),
            container_healed_count,
            heal_attempts: Arc::new(Mutex::new(HashMap::new())),
        })
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
                                let mut heal_attempts = self.heal_attempts.lock().await;

                                let counter = heal_attempts.entry(container_id.clone()).or_insert(0);
                                
                                // If heal attempts hit the threshold of 3, skip healing for the container
                                if *counter >= 0 {
                                    continue;
                                }

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

                                        self.container_healed_count.inc();
                                    }

                                    *counter += 1;
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
                    if let Err(e) = self.docker.restart_container(&container_id, None).await {
                        eprintln!("Error removing container: {}. Proceeding to restart & recreate.", e);

                        // Delete & re-reate container if restarting fails.
                        self.docker.remove_container(&container_id, Some(RemoveContainerOptions {
                            force: true,
                            ..Default::default()
                        })).await.unwrap();

                        if let Err(e) = self.docker.create_container(Some(self.create_options.clone()), self.container_config.clone()).await {
                            eprintln!("Error creating container: {}", e);
                        } else {
                            let healing_report = &mut *self.healing_report.lock().await;

                            let system_time = SystemTime::now();
                            let datetime: DateTime<Utc> = system_time.into();
                            let timestamp_str = datetime.to_rfc3339();

                            healing_report.push(HealingReport {
                                container_id: container_id.clone(),
                                timestamp: timestamp_str,
                                event: "Container was re-created".to_string(),
                            });

                            self.container_healed_count.inc();
                        }
                    } else {
                        let healing_report = &mut *self.healing_report.lock().await;

                        let system_time = SystemTime::now();
                        let datetime: DateTime<Utc> = system_time.into();
                        let timestamp_str = datetime.to_rfc3339();

                        healing_report.push(HealingReport {
                            container_id: container_id.clone(),
                            timestamp: timestamp_str,
                            event: "Container was restarted".to_string(),
                        });

                        self.container_healed_count.inc();
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

    async fn get_metrics(
        &self,
        _request: Request<Empty>,
     ) -> Result<Response<MetricsResponse>, Status> { 
            let registry = Registry::new();
            let mut buffer = vec![];
            let encoder = TextEncoder::new();
            let metrics_families = registry.gather();
            encoder.encode(&metrics_families, &mut buffer).unwrap();

            let metrics_str = String::from_utf8(buffer).unwrap();

            Ok(Response::new(MetricsResponse {
                metrics: metrics_str
         }))
    }
}