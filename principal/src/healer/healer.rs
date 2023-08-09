mod healer_proto {
    include!("../healer.rs");
}

use prometheus::{Opts, Registry, Counter, Encoder, TextEncoder};
use std::time::{Duration, SystemTime};
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;

use tonic::{Request, Response, Status};
use tokio::time;
use chrono::{DateTime, Utc};

use bollard::container::{CreateContainerOptions, RemoveContainerOptions, ListContainersOptions};
use bollard::Docker;

use healer_proto::{
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
    pub docker: Arc<Mutex<Docker>>,
    pub create_options: CreateContainerOptions<String>,
    pub container_config: bollard::container::Config<String>,
    pub healing: Arc<Mutex<bool>>,
    pub healing_report: Arc<Mutex<Vec<HealingReport>>>,
    pub container_healed_count: Counter,
    pub heal_attempts: Arc<Mutex<HashMap<String, u32>>>,
}

fn container_state_status_enum_to_str(status: &bollard::secret::ContainerStateStatusEnum) -> &str {
    match status {
        bollard::secret::ContainerStateStatusEnum::DEAD => "dead",
        _ => "unknown"
    }
}

fn container_status_to_str(status: &bollard::secret::ContainerStateStatusEnum) -> &str {
    match status {
        bollard::secret::ContainerStateStatusEnum::DEAD => "dead",
        _ => "unknown",
    }
}

impl MyHealer {

    async fn log_healing(&self, container_id: &str, _event: &str) {
        let mut healing_report = match self.healing_report.lock() {
            Ok(guard) => guard,
            Err(e) => {
                eprintln!("Error locking healing report: {}", e);
                return;
            },
        };
    
        let system_time = SystemTime::now();
        let datetime: DateTime<Utc> = system_time.into();
        let timestamp_str = datetime.to_rfc3339();
    
        healing_report.push(HealingReport {
            container_id: container_id.clone().to_owned(),
            timestamp: timestamp_str,
            event: "Container was re-created".to_string(),
        });
    
        self.container_healed_count.inc();
    }

    pub fn new(docker: Docker, create_options: CreateContainerOptions<String>, container_config: bollard::container::Config<String>, registry: &Registry) -> Result<Self, Box<dyn std::error::Error>> {
        let counter_opts = Opts::new("container_healed_count", "Number of containers healed");
        let container_healed_count = Counter::with_opts(counter_opts)?;
        registry.register(Box::new(container_healed_count.clone()))?;

        Ok(MyHealer {
            docker: Arc::new(Mutex::new(docker)),
            create_options,
            container_config,
            healing: Arc::new(Mutex::new(false)),
            healing_report: Arc::new(Mutex::new(Vec::new())),
            container_healed_count,
            heal_attempts: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn start(&self) {
        let mut healing = match self.healing.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        *healing = true;

        let mut docker_guard = self.docker.lock().unwrap();
        let mut docker = &mut *docker_guard;
    
        let mut backoff = Duration::from_secs(1);
        while *self.healing.lock().unwrap() {
            match docker.list_containers(Some(ListContainersOptions::<String> {
                all: true,
                ..Default::default()
            })).await {
                Ok(containers) => {
                    for container in &containers {
                        if let Some(container_id) = &container.id {
                            match docker.inspect_container(container_id, None).await {
                                Ok(inspect_data) => {
                                    if let Some(state) = &inspect_data.state {
                                        match container_state_status_enum_to_str(state.status.as_ref().unwrap()) {
                                            "dead" => {
                                                if let Err(e) = docker.restart_container(container_id, None).await {
                                                    eprintln!("Error removing container: {}. Proceeding to restart & recreate.", e);

                                                    docker.remove_container(container_id, Some(RemoveContainerOptions {
                                                        force: true,
                                                        ..Default::default()
                                                    })).await.unwrap();

                                                    if let Err(e) = docker.remove_container(container_id, Some(RemoveContainerOptions {
                                                        force: true,
                                                        ..Default::default()
                                                    })).await {
                                                        eprintln!("Error removing container: {}", e);
                                                    }

                                                    if let Err(e) = docker.create_container(Some(self.create_options.clone()), self.container_config.clone()).await {
                                                        eprintln!("Error creating container: {}", e);
                                                    } else {
                                                        self.log_healing(container_id, "Container was re-created").await;
                                                    }
                                                } else {
                                                    self.log_healing(container_id, "Container was restarted due to being in dead state").await;
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                                Err(e) => {
                                     eprintln!("Error inspecting container: {}", e);
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
    

    pub async fn stop(&self) {
         let mut healing = match self.healing.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        *healing = false;
    }
    
    pub async fn heal_containers(&self, containers_id: Vec<String>) {
        let mut docker_guard = self.docker.lock().unwrap();
        let docker = &mut *docker_guard;

        for container_id in &containers_id {
            let container_info = docker.inspect_container(container_id, None).await;
    
            if let Ok(info) = container_info {
                if let Some(container_state) = info.state {
                    match container_status_to_str(container_state.status.as_ref().unwrap()) {
                        "dead" => {
                            if let Err(e) = docker.restart_container(container_id, None).await {
                                eprintln!("Error restarting container: {}. Proceeding to recreate.", e);
    
                                if let Err(err) = docker.remove_container(container_id, Some(RemoveContainerOptions {
                                    force: true,
                                    ..Default::default()
                                })).await {
                                    eprintln!("Failed to remove container: {:?}", err);
                                }
    
                                if let Err(e) = docker.create_container(Some(self.create_options.clone()), self.container_config.clone()).await {
                                    eprintln!("Error creating container: {}", e);
                                } else {
                                    self.log_healing(container_id, "Container was restarted").await;
                                }
                            } else {
                                self.log_healing(container_id, "Container was healed").await;
                            }
                        }
                        _ => {
                           
                        }
                    }
                }
            } else {
                eprintln!("Error inspecting container {}: {:?}", container_id, container_info.err().unwrap());
            }
        }
    }

    pub async fn perform_update(&self, containers_id: Vec<String>) {
        let mut docker_guard = self.docker.lock().unwrap();
        let docker = &mut *docker_guard;

        for container_id in containers_id {
            if let Err(err) = docker.remove_container(&container_id, Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            })).await {
                eprintln!("Failed to remove container: {:?}", err);
            };

            if let Err(e) = docker.create_container(Some(self.create_options.clone()), self.container_config.clone()).await {
                eprintln!("Error creating container: {}", e)
            } else {
                self.log_healing(&container_id, "Container was healed");
            }

            time::sleep(Duration::from_secs(10)).await;
        }
    }
}

#[tonic::async_trait]
impl healer_proto::healer_server::Healer for MyHealer {
    async fn start_healing(
        &self,
        _request: Request<StartHealingRequest>,
    ) -> Result<Response<StartHealingResponse>, Status> {
        let healer = self.clone();

        
        tokio::task::spawn_local(async move {
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
        tokio::task::spawn_local(async move {
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
        let report = self.healing_report.lock().unwrap();
        Ok(Response::new(GetHealingReportResponse {
            healing_events: report.to_vec(),
        }))
    }

    async fn perform_rolling_update(
        &self,
        request: Request<UpdateRequest>,
    ) -> Result<Response<UpdateResponse>, Status> {
        let healer = self.clone();

        let containers_ids = request.into_inner().container_ids;
        tokio::task::spawn_local(async move {
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