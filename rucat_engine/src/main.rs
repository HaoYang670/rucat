use rucat_common::config::EngineConfig;
use rucat_common::database::DataBase;
use rucat_common::engine::EngineState::*;
use rucat_common::engine_grpc::greeter_server::{Greeter, GreeterServer};
use rucat_common::engine_grpc::{HelloReply, HelloRequest};
use rucat_common::error::RucatError;
use std::net::{Ipv6Addr, SocketAddrV6};
use tokio::io::{self, AsyncReadExt};
use tokio::net::TcpListener;
use tonic::transport::server::TcpIncoming;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{error, info};

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
    tracing_subscriber::fmt::init();

    // rucat engine should get the database endpoint and engine id from the config file
    // TODO: read from stdin directly (serde_json: from_reader)
    let EngineConfig {
        engine_id,
        db_endpoint,
    } = {
        let mut buf = vec![];
        io::stdin().read_to_end(&mut buf).await?;
        serde_json::from_slice(&buf)?
    };
    info!(
        "Received configs from server: engine_id: {:?}, db_endpoint: {}",
        engine_id, db_endpoint
    );

    // set port to 0 to let the OS choose a free port
    let addr = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 0, 0, 0);
    let listener = TcpListener::bind(addr).await?;
    let addr = listener.local_addr()?;
    info!("Rucat engine is listening on: {}", addr);

    let db = DataBase::connect_local_db(db_endpoint).await?;
    let response = db
        .update_engine_state(&engine_id, [Pending], Running, Some(addr.into()))
        .await?;
    match response {
        None => Err(RucatError::FailedToStartEngine(
            "Not register engine in the database".to_string(),
        )),
        Some(response) => {
            if response.update_success {
                info!("Engine state updated successfully");
                Ok(())
            } else {
                let err_msg = "Failed to update engine state";
                error!(err_msg);
                Err(RucatError::FailedToStartEngine(err_msg.to_string()))
            }
        }
    }?;

    // same default value of `nodelay` and `keepalive` as those in [Server]
    let tpc_incoming = TcpIncoming::from_listener(listener, false, None)
        .map_err(|err| RucatError::FailedToStartEngine(err.to_string()))?;

    let greeter = MyGreeter::default();

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve_with_incoming(tpc_incoming)
        .await
        .map_err(|err| RucatError::FailedToStartEngine(err.to_string()))?;

    Ok(())
}
