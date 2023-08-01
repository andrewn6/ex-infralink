use dotenv::dotenv;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};

pub mod providers;
pub mod shared_config;
pub mod rules;
pub mod manager;
pub mod db;
pub mod gpu;
pub mod volumes;

use volumes::volumes::{VolumeManager, HetznerVolumeConfig, VultrVolumeConfig, HetznerVolumeAttachmentConfig, HetznerVolumeResizeConfig, VultrVolumeDetachConfig, VultrVolumeAttachmentConfig};
use manager::manager::{Manager, ManagerError, AnyInstance};

use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::http::StatusCode;

type ResultResponse = Result<Response<Body>, Box<dyn Error + Send + Sync>>;

fn handle_error(err: Box<dyn std::error::Error>) -> ResultResponse {
	Ok(Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from(err.to_string())).unwrap())
}

async fn await_body_bytes(req: Request<Body>) -> Result<Vec<u8>, hyper::Error> {
    let full_body = hyper::body::to_bytes(req.into_body()).await?;
    Ok(full_body.to_vec())
}

fn get_query_param(req: &Request<Body>, key: &str) -> Option<String> {
    req.uri().query().and_then(|query| {
        let params: HashMap<String, String> = form_urlencoded::parse(query.as_bytes()).into_owned().collect();
        params.get(key).cloned()
    })
}

async fn handle_instances_request(
    req: Request<Body>,
    manager: Arc<Manager>,
) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/instances") => {
            let instances = match manager.get_instances().await {
                Ok(instances) => instances,
                Err(e) => {
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from(e.to_string()))
                        .unwrap())
                }
            };

            let response_body = format!("{:?}", instances);

            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(response_body))
                .unwrap())
        }
        _ => {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not found"))
                .unwrap())
        }
    }
}

async fn handle_request(
    req: Request<Body>,
    manager: Arc<Manager>,
    volume_manager: Arc<Mutex<VolumeManager>>,
) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
    let vm = volume_manager.lock();

	match (req.method(), req.uri().path()) {
		(&hyper::Method::GET, "/volumes/hetzner") => {
            let volume_config: HetznerVolumeConfig = match serde_json::from_slice(&await_body_bytes(req).await?) {
                Ok(config) => config,
                Err(e) => return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e.to_string())).unwrap()),
            };

            let result = volume_manager.lock().unwrap().create_volume_on_hetzner(volume_config).await;
            result.map(|volume| Response::new(Body::from(serde_json::to_string(&volume).unwrap())))
                  .or_else(handle_error)
        }

        (&hyper::Method::POST, "/volumes/hetzner/attach") => {
            let attach_config: HetznerVolumeAttachmentConfig = match serde_json::from_slice(&await_body_bytes(req).await?) {
                Ok(config) => config,
                Err(e) => return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e.to_string())).unwrap()),
            };

            let volume_id = req.uri().query().and_then(|query| {
                let params: HashMap<String, String> = form_urlencoded::parse(query.as_bytes()).into_owned().collect();
                params.get("volume_id").cloned()
            }).ok_or_else(|| "Missing volume_id query parameter".to_string());

            let volume_id = match volume_id {
                Ok(id) => id,
                Err(e) => return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e)).unwrap()),
            };

            let result = volume_manager.lock().unwrap().attach_volume_on_hetzner(&volume_id, attach_config).await;
            result.map(|volume| Response::new(Body::from(serde_json::to_string(&volume).unwrap())))
                  .or_else(handle_error)
        }

        (&hyper::Method::POST, "/volumes/hetzner/detach") => {
            let volume_id = req.uri().query().map(|query| {
                let params: HashMap<String, String> = form_urlencoded::parse(query.as_bytes()).into_owned().collect();
                params.get("volume_id").map(|id| id.to_string())
            }).flatten().ok_or_else(|| "Missing volume_id query parameter".to_string());

            let volume_id = match volume_id {
                Ok(id) => id,
                Err(e) => return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e)).unwrap()),
            };

            let result = volume_manager.lock().unwrap().detach_volume_hetzner(&volume_id).await;
            result.map(|volume| Response::new(Body::from("Volume detached successfully")))
                  .or_else(handle_error)

        }

        (&hyper::Method::POST, "/volumes/hetzner/resize") => {
            let resize_config: HetznerVolumeResizeConfig = match serde_json::from_slice(&await_body_bytes(req).await?) {
                Ok(config) => config,
                Err(e) => return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e.to_string())).unwrap()),
            };

            let volume_id = req.uri().query().map(|query| {
                let params: HashMap<String, String> = form_urlencoded::parse(query.as_bytes()).into_owned().collect();
                params.get("volume_id").map(|id| id.to_string())
            }).flatten().ok_or_else(|| "Missing volume_id query parameter".to_string()).unwrap();
                    
            let result = volume_manager.lock().unwrap().resize_volume_on_hetzner(&volume_id, resize_config).await;
            result.map(|_| Response::new(Body::from("Volume resized successfully")))
                  .or_else(handle_error)
        }

        (&hyper::Method::POST, "/volumes/vultr") => {
            let volume_config: VultrVolumeConfig = match serde_json::from_slice(&await_body_bytes(req).await?) {
                Ok(config) => config,
                Err(e) => return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e.to_string())).unwrap()),
            };

            let result = volume_manager.lock().unwrap().create_volume_on_vultr(volume_config).await;
            result.map(|volume| Response::new(Body::from(serde_json::to_string(&volume).unwrap())))
                  .or_else(handle_error)
        }   

        (&hyper::Method::POST, "/volumes/vultr/attach") => {
            let attach_config: VultrVolumeAttachmentConfig = match serde_json::from_slice(&await_body_bytes(req).await?) {
                Ok(config) => config,
                Err(e) => return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e.to_string())).unwrap()),
            };

            let volume_id = req.uri().query().and_then(|query| {
                let params: HashMap<String, String> = form_urlencoded::parse(query.as_bytes()).into_owned().collect();
                params.get("volume_id").cloned()
            });

            let volume_id = match volume_id {
                Some(id) => id,
                None => return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from("Missing volume_id query parameter")).unwrap()),
            };

            let result = volume_manager.lock().unwrap().attach_volume_on_vultr(&volume_id, attach_config).await;
            result.map(|volume| Response::new(Body::from(serde_json::to_string(&volume).unwrap())))
                  .or_else(handle_error)
        }

        (&hyper::Method::POST, "/volumes/vultr/detach") => {
            let detach_config: VultrVolumeDetachConfig = match serde_json::from_slice(&await_body_bytes(req).await?) {
                Ok(config) => config,
                Err(e) => return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e.to_string())).unwrap()),
            };

            let volume_id = get_query_param(&req, "volume_id")
                .ok_or_else(|| "Missing volume_id query parameter".to_string())?;

            let vm = volume_manager.lock().unwrap();
            vm.detach_volume_on_vultr(&volume_id, detach_config).await
                    .map(|volume| Response::new(Body::from(serde_json::to_string(&volume).unwrap())))
                    .or_else(handle_error)
        },

        (&hyper::Method::POST, "/volumes/vultr/resize") => {
            let volume_id = req.uri().query().map(|query| {
                  let params: HashMap<String, String> = form_urlencoded::parse(query.as_bytes()).into_owned().collect();
                  params.get("volume_id").map(|id| id.to_string())
              }).flatten().ok_or_else(|| "Missing volume_id query parameter".to_string()).unwrap();
          
              let new_label = req.uri().query().map(|query| {
                  let params: HashMap<String, String> = form_urlencoded::parse(query.as_bytes()).into_owned().collect();
                  params.get("new_label").map(|id| id.to_string())
              }).flatten().ok_or_else(|| "Missing new_label query parameter".to_string()).unwrap();
          
              let new_size_gb = req.uri().query().map(|query| {
                  let params: HashMap<String, String> = form_urlencoded::parse(query.as_bytes()).into_owned().collect();
                  params.get("new_size_gb").and_then(|size| size.parse::<i32>().ok())
              }).flatten().ok_or_else(|| "Missing new_size_gb query parameter".to_string()).unwrap();
              
              let vm = volume_manager.lock().unwrap();
              vm.resize_volume_on_vultr(&volume_id, &new_label, new_size_gb).await
                  .map(|_| Response::new(Body::from("Volume resized successfully")))
                  .or_else(handle_error)
          },

          (&hyper::Method::GET, "/instances") => {
            handle_instances_request(req, manager).await
          },

		_ => Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::from("Not Found")).unwrap()),
	}
}

async fn process_request(
    req: Request<Body>,
    manager_arc: Arc<Manager>,
    volume_manager: Arc<Mutex<VolumeManager>>,
) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
    handle_request(req, manager_arc, volume_manager).await
}

#[tokio::main]
async fn main() {
    dotenv().ok(); 

    let manager = Manager::new().await.expect("Failed to create manager");
    let manager_arc = Arc::new(manager);
    
    let volume_manager = Arc::new(Mutex::new(VolumeManager::volume()));

    let make_svc = make_service_fn(move |_conn| {
        let manager_clone = manager_arc.clone();
        let volume_manager_clone = volume_manager.clone(); // Add this line
        async {
            Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                let manager_inner_clone = manager_clone.clone();
                process_request(req, manager_inner_clone, volume_manager_clone.clone()) // Add volume_manager parameter
            }))
        }
    });

    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr).serve(make_svc);

    println!("Server started on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}