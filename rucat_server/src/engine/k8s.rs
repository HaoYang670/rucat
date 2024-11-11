//! Functions to manage Spark engine on k8s

use ::std::borrow::Cow;

use ::rucat_common::{engine::{
    get_spark_app_id, get_spark_driver_name, get_spark_service_name, EngineConfigs, EngineId,
}, tracing::debug};
use rucat_common::{
    error::*,
    k8s_openapi::api::core::v1::{Pod, Service},
    kube::{api::PostParams, Api, Client},
};

use serde_json::json;

const SPARK_SERVICE_SELECTOR: &str = "rucat-engine-selector";

/// Create Spark app and Spark connect server on k8s
pub(super) async fn create_engine(id: &EngineId, config: &EngineConfigs) -> Result<()> {
    let client = Client::try_default()
        .await
        .map_err(RucatError::fail_to_create_engine)?;

    let spark_app_id = get_spark_app_id(id);
    let spark_driver_name = get_spark_driver_name(id);
    let spark_service_name = get_spark_service_name(id);
    let mut args = config.to_spark_submit_format_with_preset_configs(id);
    args.extend([
        Cow::Borrowed("--master"),
        Cow::Borrowed("k8s://https://kubernetes:443"),
        Cow::Borrowed("--deploy-mode"),
        Cow::Borrowed("client"),
        Cow::Borrowed("--packages"),
        Cow::Borrowed("org.apache.spark:spark-connect_2.12:3.5.3"),
    ]);

    let pod: Pod = serde_json::from_value(json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": {
            "name": spark_driver_name,
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
                    "args": args,
                }
            ]
        }
    }))
    .map_err(RucatError::fail_to_create_engine)?;

    // Create a Pod API instance
    let pods: Api<Pod> = Api::namespaced(client.clone(), "default");

    // Create the Pod
    let pp = PostParams::default();
    let _pod = pods
        .create(&pp, &pod)
        .await
        .map_err(RucatError::fail_to_create_engine)?;
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
    }))
    .map_err(RucatError::fail_to_create_engine)?;

    // Create a Service API instance
    let services: Api<Service> = Api::namespaced(client, "default");
    // Create the Service
    let _service = services
        .create(&pp, &service)
        .await
        .map_err(RucatError::fail_to_create_engine)?;

    Ok(())
}

/// Delete Spark app and the headless service on k8s
pub(super) async fn delete_engine(id: &EngineId) -> Result<()> {
    let client = Client::try_default()
        .await
        .map_err(RucatError::fail_to_delete_engine)?;

    let spark_driver_name = get_spark_driver_name(id);
    debug!("Deleting Pod: {}", spark_driver_name);
    let spark_service_name = get_spark_service_name(id);

    // Create a Pod API instance
    let pods: Api<Pod> = Api::namespaced(client.clone(), "default");
    // Delete the Pod
    let _pod = pods
        .delete(&spark_driver_name, &Default::default())
        .await
        .map_err(RucatError::fail_to_delete_engine)?;

    // Create a Service API instance
    let services: Api<Service> = Api::namespaced(client, "default");
    // Delete the Service
    let _service = services
        .delete(&spark_service_name, &Default::default())
        .await
        .map_err(RucatError::fail_to_delete_engine)?;

    Ok(())
}
