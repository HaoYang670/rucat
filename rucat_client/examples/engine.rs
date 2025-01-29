use ::rucat_client::{resource_client, Credentials};
use ::rucat_common::engine::{CreateEngineRequest, EngineState, EngineType};
use rucat_common::tokio;

#[tokio::main]
async fn main() {
    let request = CreateEngineRequest {
        name: "spark0".to_owned(),
        engine_type: EngineType::Spark,
        version: "3.5.4".to_owned(),
        config: None,
    };
    let credentials = Credentials::Bearer { token: "admin" };
    let client = resource_client::ResourceClient::new("http://localhost:3000", Some(credentials));
    let id = client.create_engine(&request).await.unwrap();
    println!("Engine created with id: {}", id);
    loop {
        let info = client.get_engine_info(&id).await.unwrap();
        println!("Engine {} is {:?}", id, info.state);
        match info.state {
            EngineState::Running => break,
            EngineState::WaitToStart | EngineState::TriggerStart | EngineState::StartInProgress => {
            }
            other => panic!("Unexpected engine state: {:?}", other),
        };
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    client.stop_engine(&id).await.unwrap();
    println!("Stopped engine");
    loop {
        let info = client.get_engine_info(&id).await.unwrap();
        println!("Engine {} is {:?}", id, info.state);
        match info.state {
            EngineState::Terminated => break,
            EngineState::WaitToTerminate
            | EngineState::TriggerTermination
            | EngineState::TerminateInProgress => {}
            other => panic!("Unexpected engine state: {:?}", other),
        };
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    client.delete_engine(&id).await.unwrap();
    println!("Deleted engine");

    let engines = client.list_engines().await.unwrap();
    println!("Engines: {:?}", engines);
}
