mod rollout {
    include!("../rollout.rs");
}

use bollard::Docker;

use std::time::Duration;
use tokio::time::sleep;
use tonic::{Request, Response, Status};
use rollout::rollout_strategy_server::RolloutStrategy;
use rollout::{ExponentialStrategy, StartRolloutRequest, StartRolloutResponse};

use self::rollout::RolloutStrategy;
    

pub struct ExponentialServiceImpl {
    pub docker: Docker,
}

#[tonic::async_trait]
impl rollout_strategy for ExponentialServiceImpl {
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