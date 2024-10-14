//! Functions to manage Spark engine on k8s

use ::rucat_common::EngineId;
use rucat_common::error::Result;
use rucat_common::{
    k8s_openapi::api::core::v1::{Pod, Service},
    kube::{api::PostParams, Api, Client},
};

use ::tracing::debug;
use serde_json::json;

const SPARK_SERVICE_SELECTOR: &str = "rucat-engine-selector";

fn get_spark_app_id(id: &EngineId) -> String {
    format!("rucat-spark-{}", id.as_str())
}

fn get_spark_driver_name(id: &EngineId) -> String {
    format!("{}-driver", get_spark_app_id(id))
}
fn get_spark_service_name(id: &EngineId) -> String {
    get_spark_app_id(id)
}

/// Create Spark app and Spark connect server on k8s
pub(super) async fn create_engine(id: &EngineId) -> Result<()> {
    let client = Client::try_default().await?;

    let spark_app_id = get_spark_app_id(id);
    let spark_service_name = get_spark_service_name(id);
    let pod: Pod = serde_json::from_value(json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": {
            "name": get_spark_driver_name(id),
            "labels": {
                SPARK_SERVICE_SELECTOR: spark_app_id,
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
                        // NOTE: spark.app.name is always "Spark Connect Server" and cannot be modified
                        "--master", "k8s://https://kubernetes:443",
                        "--deploy-mode", "client",
                        "--conf", format!("spark.app.id={}", spark_app_id),
                        "--conf", "spark.kubernetes.container.image=apache/spark:3.5.3",
                        "--conf", "spark.executor.instances=1",
                        "--conf", format!("spark.driver.host={}", spark_service_name),
                        "--conf", format!("spark.kubernetes.executor.podNamePrefix={}", spark_app_id),
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
            "name": spark_service_name,
        },
        "spec": {
            "type": "ClusterIP",
            "clusterIP": "None",
            "selector": {
                SPARK_SERVICE_SELECTOR: spark_app_id,
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

/// Delete Spark app and Spark connect server on k8s
pub(super) async fn delete_engine(id: &EngineId) -> Result<()> {
    let client = Client::try_default().await?;

    let spark_driver_name = get_spark_driver_name(id);
    debug!("Deleting Pod: {}", spark_driver_name);
    let spark_service_name = get_spark_service_name(id);

    // Create a Pod API instance
    let pods: Api<Pod> = Api::namespaced(client.clone(), "default");
    // Delete the Pod
    let _pod = pods.delete(&spark_driver_name, &Default::default()).await?;

    // Create a Service API instance
    let services: Api<Service> = Api::namespaced(client, "default");
    // Delete the Service
    let _service = services
        .delete(&spark_service_name, &Default::default())
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_spark_app_id() {
        let id = EngineId::new("abc".to_owned());
        assert_eq!(get_spark_app_id(&id), "rucat-spark-abc");
    }

    #[test]
    fn test_get_spark_driver_name() {
        let id = EngineId::new("abc".to_owned());
        assert_eq!(get_spark_driver_name(&id), "rucat-spark-abc-driver");
    }

    #[test]
    fn test_get_spark_service_name() {
        let id = EngineId::new("abc".to_owned());
        assert_eq!(get_spark_service_name(&id), "rucat-spark-abc");
    }
}
