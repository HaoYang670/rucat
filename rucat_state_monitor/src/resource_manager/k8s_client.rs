use ::std::{borrow::Cow, collections::BTreeMap};

use ::k8s_openapi::api::core::v1::{Pod, Service};
use ::kube::{api::PostParams, Api, Client};
use ::rucat_common::{
    anyhow::anyhow,
    engine::{
        get_spark_app_id, get_spark_driver_name, get_spark_service_name, EngineConfigs, EngineId,
        EngineState,
    },
    error::{Result, RucatError},
    serde_json::{self, json},
    tracing::{debug, warn},
};

use super::{ResourceManager, ResourceState};

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

    // convert engine configurations to Spark submit format
    fn to_spark_submit_format(
        id: &EngineId,
        user_configs: &EngineConfigs,
    ) -> Result<Vec<Cow<'static, str>>> {
        // Preset configurations for Spark on Kubernetes.
        // Users are not allowed to set these configurations.
        // make the map ordered for easier testing
        let preset_configs = BTreeMap::from([
            (Cow::Borrowed("spark.app.id"), get_spark_app_id(id)),
            (
                Cow::Borrowed("spark.driver.extraJavaOptions"),
                Cow::Borrowed("-Divy.cache.dir=/tmp -Divy.home=/tmp"),
            ),
            (
                Cow::Borrowed("spark.driver.host"),
                get_spark_service_name(id),
            ),
            (
                Cow::Borrowed("spark.kubernetes.container.image"),
                Cow::Borrowed("apache/spark:3.5.3"),
            ),
            (
                Cow::Borrowed("spark.kubernetes.driver.pod.name"),
                get_spark_driver_name(id),
            ),
            (
                Cow::Borrowed("spark.kubernetes.executor.podNamePrefix"),
                get_spark_app_id(id),
            ),
        ]);

        match preset_configs
            .keys()
            .find(|k| user_configs.contains_key(*k))
        {
            Some(key) => Err(RucatError::not_allowed(anyhow!(
                "The config {} is not allowed as it is reserved.",
                key
            ))),
            None => Ok([
                Cow::Borrowed("--master"),
                Cow::Borrowed("k8s://https://kubernetes:443"),
                Cow::Borrowed("--deploy-mode"),
                Cow::Borrowed("client"),
                Cow::Borrowed("--packages"),
                Cow::Borrowed("org.apache.spark:spark-connect_2.12:3.5.3"),
            ]
            .iter()
            .cloned()
            .chain(
                preset_configs
                    .iter()
                    .chain(user_configs.iter())
                    .flat_map(|(k, v)| {
                        [Cow::Borrowed("--conf"), Cow::Owned(format!("{}={}", k, v))]
                    }),
            )
            .collect()),
        }
    }

    pub async fn new() -> Result<Self> {
        let client = Client::try_default()
            .await
            .map_err(RucatError::fail_to_delete_engine)?;
        Ok(Self { client })
    }
}

impl ResourceManager for K8sClient {
    type ResourceState = K8sPodState;

    async fn create_resource(&self, id: &EngineId, configs: &EngineConfigs) -> Result<()> {
        let spark_app_id = get_spark_app_id(id);
        let spark_driver_name = get_spark_driver_name(id);
        let spark_service_name = get_spark_service_name(id);
        let args = Self::to_spark_submit_format(id, configs)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    fn check_preset_config(key: &'static str) {
        let config = BTreeMap::from([(Cow::Borrowed(key), Cow::Borrowed(""))]);
        let id = EngineId::new(Cow::Borrowed("123")).unwrap();
        let result = K8sClient::to_spark_submit_format(&id, &config);
        assert!(result.is_err_and(|e| e.to_string().starts_with(&format!(
            "Not allowed: The config {} is not allowed as it is reserved.",
            key
        ))));
    }

    #[test]
    fn preset_configs_are_not_allowed_to_be_set() {
        check_preset_config("spark.app.id");
        check_preset_config("spark.driver.extraJavaOptions");
        check_preset_config("spark.driver.host");
        check_preset_config("spark.kubernetes.container.image");
        check_preset_config("spark.kubernetes.driver.pod.name");
        check_preset_config("spark.kubernetes.executor.podNamePrefix");
    }

    #[test]
    fn empty_engine_config() -> Result<()> {
        let spark_submit_format =
            K8sClient::to_spark_submit_format(&EngineId::try_from("abc")?, &BTreeMap::new())?;
        assert_eq!(
            spark_submit_format,
            vec![
                "--master",
                "k8s://https://kubernetes:443",
                "--deploy-mode",
                "client",
                "--packages",
                "org.apache.spark:spark-connect_2.12:3.5.3",
                "--conf",
                "spark.app.id=rucat-spark-abc",
                "--conf",
                "spark.driver.extraJavaOptions=-Divy.cache.dir=/tmp -Divy.home=/tmp",
                "--conf",
                "spark.driver.host=rucat-spark-abc",
                "--conf",
                "spark.kubernetes.container.image=apache/spark:3.5.3",
                "--conf",
                "spark.kubernetes.driver.pod.name=rucat-spark-abc-driver",
                "--conf",
                "spark.kubernetes.executor.podNamePrefix=rucat-spark-abc",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
        Ok(())
    }

    #[test]
    fn engine_config_with_1_item() -> Result<()> {
        let configs = BTreeMap::from([(
            Cow::Borrowed("spark.executor.instances"),
            Cow::Borrowed("2"),
        )]);

        let spark_submit_format =
            K8sClient::to_spark_submit_format(&EngineId::try_from("abc")?, &configs)?;
        assert_eq!(
            spark_submit_format,
            vec![
                "--master",
                "k8s://https://kubernetes:443",
                "--deploy-mode",
                "client",
                "--packages",
                "org.apache.spark:spark-connect_2.12:3.5.3",
                "--conf",
                "spark.app.id=rucat-spark-abc",
                "--conf",
                "spark.driver.extraJavaOptions=-Divy.cache.dir=/tmp -Divy.home=/tmp",
                "--conf",
                "spark.driver.host=rucat-spark-abc",
                "--conf",
                "spark.kubernetes.container.image=apache/spark:3.5.3",
                "--conf",
                "spark.kubernetes.driver.pod.name=rucat-spark-abc-driver",
                "--conf",
                "spark.kubernetes.executor.podNamePrefix=rucat-spark-abc",
                "--conf",
                "spark.executor.instances=2",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
        Ok(())
    }
}
