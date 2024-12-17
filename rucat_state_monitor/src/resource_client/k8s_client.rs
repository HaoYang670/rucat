use ::std::borrow::Cow;

use ::k8s_openapi::api::core::v1::{Pod, Service};
use ::kube::{api::PostParams, Api, Client};
use ::rucat_common::{
    engine::{
        get_spark_app_id, get_spark_driver_name, get_spark_service_name, EngineConfigs, EngineId,
        EngineState,
    },
    error::{Result, RucatError},
    serde_json::{self, json},
    tracing::{debug, warn},
};

use super::{ResourceClient, ResourceState};

/// Derive from K8s pod phase: https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#pod-phase
#[derive(Debug)]
pub enum K8sPodState {
    /// This is for the case when pod does not exists which is actually not a valid state in K8s.
    /// We define it to avoid using `Option<K8sPodState>` in `ResourceState`.
    NotExisted,
    Pending,
    Running,
    Succeeded,
    Failed,
    Unknown,
}

impl K8sPodState {
    pub fn from_phase(phase: Option<String>) -> Self {
        match phase.as_deref() {
            Some("Pending") => Self::Pending,
            Some("Running") => Self::Running,
            Some("Succeeded") => Self::Succeeded,
            Some("Failed") => Self::Failed,
            Some("Unknown") | None => Self::Unknown,
            // panic here because it should never happen
            Some(phase) => unreachable!("Unknown phase: {}", phase),
        }
    }
}

impl ResourceState for K8sPodState {
    fn get_new_engine_state(&self, old_state: &EngineState) -> Option<EngineState> {
        // TODO: wrap `Running` and `*InProgress` states in a new type
        match (old_state, self) {
            (EngineState::StartInProgress, Self::Pending | Self::Unknown) => None,
            (EngineState::StartInProgress, Self::Running) => Some(EngineState::Running),
            (EngineState::StartInProgress, Self::Succeeded | Self::Failed | Self::NotExisted) => {
                Some(EngineState::ErrorClean("Engine fails to start.".to_owned()))
            }

            (EngineState::Running, Self::Pending) => Some(EngineState::ErrorCleanInProgress(
                "Engine restarts unexpected.".to_owned(),
            )),
            (EngineState::Running, Self::Running | Self::Unknown) => None,
            (EngineState::Running, Self::Succeeded | Self::Failed | Self::NotExisted) => Some(
                EngineState::ErrorClean("Engine terminates during running.".to_owned()),
            ),

            (EngineState::TerminateInProgress, Self::NotExisted) => Some(EngineState::Terminated),
            (EngineState::TerminateInProgress, _) => None,

            (EngineState::ErrorCleanInProgress(s), Self::NotExisted) => {
                Some(EngineState::ErrorClean(s.clone()))
            }
            (EngineState::ErrorCleanInProgress(_), _) => None,
            (s, _) => {
                unreachable!("State {:?} should not be updated by resource state.", s);
            }
        }
    }
}

/// Client to interact with the Kubernetes cluster.
pub struct K8sClient {
    client: Client,
}

impl K8sClient {
    const SPARK_SERVICE_SELECTOR: &str = "rucat-engine-selector";

    pub async fn new() -> Result<Self> {
        let client = Client::try_default()
            .await
            .map_err(RucatError::fail_to_delete_engine)?;
        Ok(Self { client })
    }
}

impl ResourceClient for K8sClient {
    type ResourceState = K8sPodState;

    async fn create_resource(&self, id: &EngineId, config: &EngineConfigs) -> Result<()> {
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
                    Self::SPARK_SERVICE_SELECTOR: spark_app_id,
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
        .map_err(RucatError::fail_to_start_engine)?;

        // Create a Pod API instance
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), "default");

        // Create the Pod
        let pp = PostParams::default();
        let _pod = pods
            .create(&pp, &pod)
            .await
            .map_err(RucatError::fail_to_start_engine)?;
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
                    Self::SPARK_SERVICE_SELECTOR: spark_app_id,
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
        .map_err(RucatError::fail_to_start_engine)?;

        // Create a Service API instance
        let services: Api<Service> = Api::namespaced(self.client.clone(), "default");
        // Create the Service
        let _service = services
            .create(&pp, &service)
            .await
            .map_err(RucatError::fail_to_start_engine)?;

        Ok(())
    }

    async fn get_resource_state(&self, id: &EngineId) -> Self::ResourceState {
        let spark_driver_name = get_spark_driver_name(id);
        // Create a Pod API instance
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), "default");
        // Get the Pod phase
        pods.get_opt(&spark_driver_name)
            .await
            .map(|pod| {
                let state = pod.map_or(K8sPodState::NotExisted, |pod| {
                    K8sPodState::from_phase(pod.status.and_then(|s| s.phase))
                });
                debug!("Get Pod: {} state: {:?}", spark_driver_name, state);
                state
            })
            .unwrap_or_else(|e| {
                warn!(
                    "Failed to get Pod: {} due to {}, mark it state as UnKnown.",
                    spark_driver_name, e
                );
                K8sPodState::Unknown
            })
    }

    async fn clean_resource(&self, id: &EngineId) -> Result<()> {
        let spark_driver_name = get_spark_driver_name(id);
        debug!("Deleting Pod: {}", spark_driver_name);
        let spark_service_name = get_spark_service_name(id);

        // Create a Pod API instance
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), "default");
        // Delete the Pod
        let _pod = pods
            .delete(&spark_driver_name, &Default::default())
            .await
            .map_err(RucatError::fail_to_delete_engine)?;

        // Create a Service API instance
        let services: Api<Service> = Api::namespaced(self.client.clone(), "default");
        // Delete the Service
        let _service = services
            .delete(&spark_service_name, &Default::default())
            .await
            .map_err(RucatError::fail_to_delete_engine)?;

        Ok(())
    }
}
