use std::net::{Ipv6Addr, SocketAddrV6};
use tokio::net::TcpListener;
use tonic::transport::server::TcpIncoming;
use tonic::{transport::Server, Request, Response, Status};

use rucat_common::engine_grpc::greeter_server::{Greeter, GreeterServer};
use rucat_common::engine_grpc::{HelloReply, HelloRequest};
use rucat_common::error::RucatError;

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> rucat_common::error::Result<()> {
    // set port to 0 to let the OS choose a free port
    let addr = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 0, 0, 0);
    let listener = TcpListener::bind(addr).await?;
    let addr = listener.local_addr()?;

    println!("Rucat engine is listening on: {}", addr);

    // same default value of `nodelay` and `keepalive`` as those in `Server``
    let tinc = TcpIncoming::from_listener(listener, false, None)
        .map_err(|err| RucatError::FailedToStartEngine(err.to_string()))?;

    let greeter = MyGreeter::default();

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve_with_incoming(tinc)
        .await
        .map_err(|err| RucatError::FailedToStartEngine(err.to_string()))?;

    println!("Hello, world from rucat engine!");

    Ok(())
}
