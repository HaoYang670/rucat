//! RPC between server and engine.
//!
//! Create a new process for the engine and start the gRPC server.
//! The engine will be listening on the given port. (localhost for now)

use rucat_common::{config::EngineConfig, error::Result};
use rucat_common::{
    k8s_openapi::api::core::v1::{Pod, Service},
    kube::{api::PostParams, Api, Client},
};

use serde_json::json;

/// Create Spark app and Spark connect server on k8s
pub(super) async fn create_engine(config: EngineConfig) -> Result<()> {
    // Create a Kubernetes client
    let client = Client::try_default().await?;

    // Define your Pod manifest
    let spark_app_name = format!("rucat-spark-{}", config.engine_id.as_str());
    let selector = "rucat-engine-selector";
    // todo: define service account
    let pod: Pod = serde_json::from_value(json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": {
            "name": format!("{}-driver", spark_app_name),
            "labels": {
                selector: spark_app_name,
            },
        },
        "spec": {
            "restartPolicy": "Never",
            "containers": [
                {
                    "name": "spark-driver",
                    "image": "apache/spark:3.5.3",
                    "ports": [
                        { "containerPort": 4040 },
                        { "containerPort": 7078 },
                        { "containerPort": 7079 },
                        { "containerPort": 15002 },
                    ],
                    "env": [
                        // let connect server run in the foreground
                        {"name": "SPARK_NO_DAEMONIZE", "value": "true"}
                    ],
                    "command": ["/opt/spark/sbin/start-connect-server.sh"],
                    "args": [
                        "--master", "k8s://https://kubernetes:443",
                        "--deploy-mode", "client",
                        "--conf", format!("spark.app.id={}", spark_app_name),
                        "--conf", "spark.app.name=Rucat-Spark-Engine",
                        "--conf", "spark.kubernetes.container.image=apache/spark:3.5.3",
                        "--conf", "spark.executor.instances=1",
                        "--conf", format!("spark.driver.host={}", spark_app_name),
                        "--conf", format!("spark.kubernetes.executor.podNamePrefix={}", spark_app_name),
                        "--conf", "spark.driver.extraJavaOptions=-Divy.cache.dir=/tmp -Divy.home=/tmp",
                        "--packages", "org.apache.spark:spark-connect_2.12:3.5.3"],
                }
            ]
        }
    }))?;

    // Create a Pod API instance
    let pods: Api<Pod> = Api::namespaced(client.clone(), "default");

    // Create the Pod
    let pp = PostParams::default();
    let _pod = pods.create(&pp, &pod).await?;

    // Define your Headless Service manifest
    let service: Service = serde_json::from_value(json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": {
            "name": spark_app_name,
        },
        "spec": {
            "type": "ClusterIP",
            "clusterIP": "None",
            "selector": {
                selector: spark_app_name,
            },
            "ports": [
                {
                    "protocol": "TCP",
                    "port": 4040,
                    "targetPort": 4040,
                    "name": "spark-ui",
                },
                {
                    "protocol": "TCP",
                    "port": 7078,
                    "targetPort": 7078,
                    "name": "driver-rpc-port",
                },
                {
                    "protocol": "TCP",
                    "port": 7079,
                    "targetPort": 7079,
                    "name": "block-manager",
                },
                {
                    "protocol": "TCP",
                    "port": 15002,
                    "targetPort": 15002,
                    "name": "spark-connect",
                },
            ]
        }
    }))?;

    // Create a Service API instance
    let services: Api<Service> = Api::namespaced(client, "default");

    // Create the Service
    let _service = services.create(&pp, &service).await?;
    Ok(())
}
