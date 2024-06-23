use clap::Parser;
use std::net::{Ipv6Addr, SocketAddr, SocketAddrV6};
use tonic::{transport::Server, Request, Response, Status};

use rucat_common::engine_grpc::greeter_server::{Greeter, GreeterServer};
use rucat_common::engine_grpc::{HelloReply, HelloRequest};

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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IPv6 address of the engine
    #[arg(long, default_value_t = Ipv6Addr::LOCALHOST)]
    ip: Ipv6Addr,

    /// Port of the engine binding
    #[arg(long)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args { ip, port } = Args::parse();
    let addr = SocketAddrV6::new(ip, port, 0, 0);
    let greeter = MyGreeter::default();

    println!("start from rucat engine!");

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(SocketAddr::V6(addr))
        .await?;

    println!("Hello, world from rucat engine!");

    Ok(())
}
