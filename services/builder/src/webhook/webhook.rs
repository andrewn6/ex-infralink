use hyper::{Body, Request, Response, Server, StatusCode, Method};
use hyper::service::{make_service_fn, service_fn};
use serde::Deserialize;
use std::net::SocketAddr;
use hmac::{Hmac, Mac};
use crypto_mac::NewMac;
use sha2::Sha256;
use dotenv_codegen::dotenv;

type HmacSha256 = Hmac<Sha256>;

const WEBHOOK_SECRET: &str = dotenv!("GITHUB_WEBHOOK_SECRET");

#[derive(Debug, Deserialize)]
pub struct WebhookPayload {
  #[serde(rename = "ref")]
  pub ref_field: String,
  pub before: String,
  pub after: String,
  pub commits: Vec<Commit>,
}

#[derive(Debug, Deserialize)]
pub struct Commit {
    pub id: String,
    pub message: String,
    pub url: String,
    pub distinct: bool,
}

async fn handle_webhook(payload: WebhookPayload) {
    let builder_endpoint = "http://localhost:8084/build";
    let client = reqwest::blocking::Client::new();
    tokio::spawn(async move {
        match client.get(builder_endpoint).send() {
            Ok(_) => println!("Successfully pinged the builder."),
            Err(_) => println!("Failed to ping the builder"),
        }
    });
}

pub async fn handle_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/webhook") => {
            let whole_body = hyper::body::to_bytes(req.into_body()).await?;
            
            let signature = req.headers().get("X-Hub-Signature-256");

            let mut mac = HmacSha256::new_varkey(WEBHOOK_SECRET.as_bytes())
                .expect("HMAC can take key of any size");

            mac.update(&whole_body);
            let result = mac.finalize();
            let code_bytes = result.into_bytes();

            if let Some(signature) = signature {
                let (_, hex_signature) = signature.to_str().unwrap().split_at(7);
                let signature_bytes = hex::decode(hex_signature).unwrap();
                if !code_bytes.eq(&signature_bytes) {
                    return Ok(Response::builder()
                        .status(StatusCode::FORBIDDEN)
                        .body(Body::from("Invalid signature"))
                        .unwrap());
                }
            } else {
                return Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Body::from("Invalid signature"))
                    .unwrap());
            }

            let payload: WebhookPayload = serde_json::from_slice(&whole_body).unwrap();
            handle_webhook(payload).await;

            Ok(Response::new(Body::from("Webhook receiver")))


        },
        _ => {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not found"))
                .unwrap())
        }        
    }
}

pub async fn webhook_route(addr: SocketAddr) {
    let make_service = make_service_fn(|_conn| async {
        Ok::<_, hyper::Error>(service_fn(handle_request))
    });

    let server = Server::bind(&addr).serve(make_service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}