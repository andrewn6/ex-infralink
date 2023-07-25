pub mod providers;
pub mod shared_config;
use std::convert::Infallible;
use std::net::SocketAddr;

use dotenv::dotenv;

pub mod rules;
pub mod manager;
pub mod db;
pub mod gpu;
pub mod volumes;

use volumes::volumes::VolumeManager;

use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::http::StatusCode;

async fn hello_world(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
	Ok(Response::new("Hello, World".into()))
}



#[tokio::main]
async fn main() {
    let make_svc = make_service_fn(|_conn| {
        let volume_manager = VolumeManager::volume();

        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                handle_request(req, volume_manager.clone())
            }))
        }
    });

    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr).serve(make_svc);

	dotenv().unwrap();

    println!("Server started on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn handle_request(req: Request<Body>, volume_manager: VolumeManager) -> Result<Response<Body>, hyper::Error> {
	match (req.method(), req.uri().path()) {
		(&hyper::Method::GET, "/volumes/hetzner") => volume_manager.get_all_volumes_hetzner().await.map(|volumes| {
			Response::new(Body::from(serde_json::to_string(&volumes).unwrap()))
		}).or_else(handle_error),

		(&hyper::Method::GET, "/volumes/vultr")  => volume_manager.get_all_volumes_on_vultr().await.map(|volumes| {
			Response::new(Body::from(serde_json::to_string(&volumes).unwrap()))
		}).or_else(handle_error),


		_ => Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::from("Not Found")).unwrap()),
	}
}

fn handle_error(err: Box<dyn std::error::Error>) -> Result<Response<Body>, hyper::Error> {
	Ok(Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from(err.to_string())).unwrap())
}