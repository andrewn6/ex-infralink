use dotenv::dotenv;
use std::collections::HashMap;

pub mod providers;
pub mod shared_config;
pub mod rules;
pub mod manager;
pub mod db;
pub mod gpu;
pub mod volumes;

use volumes::volumes::{VolumeManager, HetznerVolumeConfig, VultrVolumeConfig, HetznerVolumeAttachmentConfig, HetznerVolumeResizeConfig, VultrVolumeDetachConfig, VultrVolumeAttachmentConfig};

use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::http::StatusCode;

fn handle_error(err: Box<dyn std::error::Error>) -> Result<Response<Body>, hyper::Error> {
	Ok(Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from(err.to_string())).unwrap())
}

async fn await_body_bytes(req: Request<Body>) -> Result<Vec<u8>, hyper::Error> {
    let full_body = hyper::body::to_bytes(req.into_body()).await?;
    Ok(full_body.to_vec())
}

async fn handle_request(req: Request<Body>, volume_manager: VolumeManager) -> Result<Response<Body>, hyper::Error> {
	match (req.method(), req.uri().path()) {
		(&hyper::Method::GET, "/volumes/hetzner") => {
            let volume_config: HetznerVolumeConfig = match serde_json::from_slice(&await_body_bytes(req).await?) {
                Ok(config) => config,
                Err(e) => return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e.to_string())).unwrap()),
            };

            volume_manager.create_volume_on_hetzner(volume_config).await  
                .map(|volume| Response::new(Body::from(serde_json::to_string(&volume).unwrap())))
                .or_else(handle_error)
        }

        (&hyper::Method::POST, "/volumes/hetzner/attach") => {
            let cloned_req = req.clone();
            let attach_config: HetznerVolumeAttachmentConfig = match serde_json::from_slice(&await_body_bytes(cloned_req).await?) {
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

            volume_manager.attach_volume_on_hetzner(&volume_id, attach_config).await
                .map(|volume| Response::new(Body::from(serde_json::to_string(&volume).unwrap())))
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

            volume_manager.detach_volume_hetzner(&volume_id).await
                .map(|volume| Response::new(Body::from("Volume detached successfully")))
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
        
            let new_size_gb = req.uri().query().map(|query| {
                let params: HashMap<String, String> = form_urlencoded::parse(query.as_bytes()).into_owned().collect();
                params.get("new_size_gb").and_then(|size| size.parse::<i32>().ok())
            }).flatten().ok_or_else(|| "Missing new_size_gb query parameter".to_string()).unwrap();
            
            volume_manager.resize_volume_on_hetzner(&volume_id, resize_config).await
                .map(|_| Response::new(Body::from("Volume resized successfully")))
                .or_else(handle_error)
        }

        (&hyper::Method::POST, "/volumes/vultr") => {
            let volume_config: VultrVolumeConfig = match serde_json::from_slice(&await_body_bytes(req).await?) {
                Ok(config) => config,
                Err(e) => return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e.to_string())).unwrap()),
            };

            volume_manager.create_volume_on_vultr(volume_config).await
                .map(|volume| Response::new(Body::from(serde_json::to_string(&volume).unwrap())))
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

            volume_manager.attach_volume_on_vultr(&volume_id, attach_config).await
                .map(|volume| Response::new(Body::from("Volume attached successfully")))
                .or_else(handle_error)
        }

        (&hyper::Method::POST, "/volumes/vultr/detach") => {
            let detach_config: VultrVolumeDetachConfig = match serde_json::from_slice(&await_body_bytes(req).await?) {
                Ok(config) => config,
                Err(e) => return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e.to_string())).unwrap()),
            };

            let volume_id = req.uri().query().map(|query| {
                let params: HashMap<String, String> = form_urlencoded::parse(query.as_bytes()).into_owned().collect();
                params.get("volume_id").map(|id| id.to_string())
            }).flatten().ok_or_else(|| "Missing volume_id query parameter".to_string());

            let volume_id = match volume_id {
                Ok(id) => id,
                Err(e) => return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e)).unwrap()),
            };

            volume_manager.detach_volume_on_vultr(&volume_id, detach_config).await
                .map(|volume| Response::new(Body::from("Volume detached successfully")))
                .or_else(handle_error)
        }

        (&hyper::Method::POST, "/volumes/vultr/resize") => {
            let resize_config: VultrVolumeDetachConfig = match serde_json::from_slice(&await_body_bytes(req).await?) {
                Ok(config) => config,
                Err(e) => return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e.to_string())).unwrap()),
            };
        
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
        
            volume_manager.resize_volume_on_vultr(&volume_id, &new_label, new_size_gb).await
                .map(|_| Response::new(Body::from("Volume resized successfully")))
                .or_else(handle_error)
        }

		_ => Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::from("Not Found")).unwrap()),
	}
}

#[tokio::main]
async fn main() {
    let make_svc = make_service_fn(|_conn| {
        let volume_manager = VolumeManager::volume();

        async {
            Ok::<_, hyper::Error>(service_fn(move |req| handle_request(req, volume_manager.clone())))
        }
    });

    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr).serve(make_svc);

    dotenv().ok(); 

    println!("Server started on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}