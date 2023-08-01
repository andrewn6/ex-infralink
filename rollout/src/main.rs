use tonic::{transport::Server, Request, Response, Status};

pub mod strategies;

#[derive(Debug)]
pub struct HelloRequest {
    pub name: String,
}

#[derive(Debug)]
pub struct HelloResponse {
    pub message: String,
}

#[tonic::async_trait]
pub trait HelloWorld {
    async fn say_hello(&self, request: Request<HelloRequest>) -> Result<Response<HelloResponse>, Status>;
}

pub struct HelloWorldService;

#[tonic::async_trait]
impl HelloWorld for HelloWorldService {
    async fn say_hello(&self, request: Request<HelloRequest>) -> Result<Response<HelloResponse>, Status> {
        let name = request.into_inner().name;
        let response = format!("Hello, {}!", name);

        Ok(Response::new(HelloResponse { message: response }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse().unwrap();
    let hello_world_service = HelloWorldService;

    println!("gRPC server listening on {}", addr);

    Server::builder()
        .add_service(HelloWorldService::new(hello_world_service))
        .serve(addr)
        .await?;

    Ok(())
}
