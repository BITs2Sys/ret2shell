use std::collections::BTreeMap;

use axum::extract::ws::WebSocket;
use chrono::{DateTime, Utc};
use futures_io::AsyncBufRead;
use k8s_openapi::{
  api::{
    core::v1::{
      ConfigMap, Container, ContainerPort, Namespace, Node, Pod, PodSpec, ResourceRequirements,
    },
    networking::v1::NetworkPolicy,
  },
  apimachinery::pkg::{api::resource::Quantity, version::Info},
};
use kube::{
  api::{ListParams, LogParams, ObjectList, ObjectMeta},
  runtime::reflector::Lookup,
  Api, Client,
};
use tokio_util::codec::Framed;

use r2s_config::cluster::ChallengeEnv;

use super::traits::ClusterError;

#[derive(Clone)]
pub struct Cluster {
  client: Option<Client>,
  namespace: Option<String>,
}

macro_rules! with_namespace {
  ($ns: expr, $reason: expr) => {
    $ns
      .clone()
      .ok_or(ClusterError::NeedNamespace($reason.to_owned()))
  };
}

macro_rules! check_enabled {
  ($client: expr) => {
    if let Some(c) = $client.clone() {
      Ok(c)
    } else {
      Err(super::traits::ClusterError::ClusterDisabled)
    }
  };
}

impl Cluster {
  pub fn new(client: Option<Client>) -> Self {
    Self {
      client,
      namespace: Some(String::from("default")),
    }
  }

  /// Set the namespace for the cluster
  ///
  /// Example:
  ///
  /// ```rust
  /// cluster.at("challenge").some_operations().await;
  /// ```
  pub fn at(&self, namespace: &str) -> Self {
    Self {
      namespace: Some(namespace.to_owned()),
      ..self.to_owned()
    }
  }

  pub async fn version(&self) -> Result<Info, ClusterError> {
    let client = check_enabled!(self.client)?;
    let version = client.apiserver_version().await?;
    Ok(version)
  }

  pub async fn nodes(&self) -> Result<ObjectList<Node>, ClusterError> {
    let client = check_enabled!(self.client)?;
    let api: Api<Node> = Api::all(client);
    let nodes = api.list(&ListParams::default()).await?;
    Ok(nodes)
  }

  pub async fn namespaces(&self) -> Result<ObjectList<Namespace>, ClusterError> {
    let client = check_enabled!(self.client)?;
    let api: Api<Namespace> = Api::all(client);
    let namespaces = api.list(&ListParams::default()).await?;
    Ok(namespaces)
  }

  pub async fn configs(&self) -> Result<ObjectList<ConfigMap>, ClusterError> {
    let client = check_enabled!(self.client)?;
    let api: Api<ConfigMap> = Api::all(client);
    let configs = api.list(&ListParams::default()).await?;
    Ok(configs)
  }

  pub async fn logs(
    &self, pod: String, container: Option<String>, follow: bool, tail_lines: Option<i64>,
    since_time: Option<DateTime<Utc>>,
  ) -> Result<impl AsyncBufRead, ClusterError> {
    let client = check_enabled!(self.client)?;
    let pods: Api<Pod> = Api::namespaced(client, &with_namespace!(&self.namespace, "logs")?);
    let logs = pods
      .log_stream(
        &pod,
        &LogParams {
          follow,
          container,
          tail_lines,
          since_time,
          timestamps: true,
          ..LogParams::default()
        },
      )
      .await?;
    Ok(logs)
  }

  pub async fn create_namespace(&self, name: &str) -> Result<Namespace, ClusterError> {
    let client = check_enabled!(self.client)?;
    let api: Api<Namespace> = Api::all(client);
    let namespace = Namespace {
      metadata: ObjectMeta {
        name: Some(name.to_owned()),
        ..Default::default()
      },
      ..Default::default()
    };
    let namespace = api.create(&Default::default(), &namespace).await?;
    Ok(namespace)
  }

  pub async fn get_network_policies(&self) -> Result<ObjectList<NetworkPolicy>, ClusterError> {
    let client = check_enabled!(self.client)?;
    let api: Api<NetworkPolicy> = Api::namespaced(
      client,
      &with_namespace!(&self.namespace, "get network policies")?,
    );
    let policies = api.list(&ListParams::default()).await?;
    Ok(policies)
  }

  pub async fn get_network_policy(
    &self, name: &str,
  ) -> Result<Option<NetworkPolicy>, ClusterError> {
    let client = check_enabled!(self.client)?;
    let api: Api<NetworkPolicy> = Api::namespaced(
      client,
      &with_namespace!(&self.namespace, "get network policy")?,
    );
    let policy = api.get_opt(name).await?;
    Ok(policy)
  }

  pub async fn create_network_policy(
    &self, policy: NetworkPolicy,
  ) -> Result<NetworkPolicy, ClusterError> {
    let client = check_enabled!(self.client)?;
    let api: Api<NetworkPolicy> = Api::namespaced(
      client,
      &with_namespace!(&self.namespace, "create network policy")?,
    );
    let policy = api.create(&Default::default(), &policy).await?;
    Ok(policy)
  }

  pub async fn delete_network_policy(&self, name: &str) -> Result<(), ClusterError> {
    let client = check_enabled!(self.client)?;
    let api: Api<NetworkPolicy> = Api::namespaced(
      client,
      &with_namespace!(&self.namespace, "delete network policy")?,
    );
    api.delete(name, &Default::default()).await?;
    Ok(())
  }

  pub async fn create_pod(&self, pod: Pod) -> Result<Pod, ClusterError> {
    let client = check_enabled!(self.client)?;
    let api: Api<Pod> = Api::namespaced(client, &with_namespace!(&self.namespace, "create pod")?);
    let pod = api.create(&Default::default(), &pod).await?;
    Ok(pod)
  }

  pub async fn delete_pod(&self, name: &str) -> Result<(), ClusterError> {
    let client = check_enabled!(self.client)?;
    let api: Api<Pod> = Api::namespaced(client, &with_namespace!(&self.namespace, "delete pod")?);
    api.delete(name, &Default::default()).await?;
    Ok(())
  }

  pub async fn get_pod(&self, name: &str) -> Result<Pod, ClusterError> {
    let client = check_enabled!(self.client)?;
    let api: Api<Pod> = Api::namespaced(client, &with_namespace!(&self.namespace, "infer pod")?);
    let pod = api.get(name).await?;
    Ok(pod)
  }

  pub async fn get_pods_by_label(&self, label: &str) -> Result<Vec<Pod>, ClusterError> {
    let client = check_enabled!(self.client)?;
    let api: Api<Pod> = Api::namespaced(client, &with_namespace!(&self.namespace, "infer pod")?);
    let pod = api
      .list(&ListParams {
        label_selector: Some(label.to_owned()),
        field_selector: Some("status.phase!=Succeeded,status.phase!=Failed".to_owned()),
        ..Default::default()
      })
      .await?;
    Ok(pod.items)
  }

  pub async fn list_pods(&self) -> Result<ObjectList<Pod>, ClusterError> {
    let client = check_enabled!(self.client)?;
    let api: Api<Pod> = Api::namespaced(client, &with_namespace!(&self.namespace, "list pods")?);
    let pods = api.list(&ListParams::default()).await?;
    Ok(pods)
  }

  pub async fn create_challenge_env(
    &self, labels: BTreeMap<String, String>, annotations: BTreeMap<String, String>,
    env_config: ChallengeEnv,
  ) -> Result<(), ClusterError> {
    let challenge_id = labels
      .get("challenge_id")
      .ok_or(ClusterError::MissingField("challenge_id".to_string()))?;
    let user_id = labels
      .get("user_id")
      .ok_or(ClusterError::MissingField("user_id".to_string()))?;
    let pod_name = format!("ret2shell-{challenge_id}-{user_id}");
    let pod = Pod {
      metadata: ObjectMeta {
        name: Some(pod_name.clone()),
        labels: Some(labels),
        annotations: Some(annotations),
        ..Default::default()
      },
      spec: Some(PodSpec {
        containers: env_config
          .images
          .iter()
          .map(|image| Container {
            name: image.name.clone(),
            image: Some(image.tag.clone()),
            ports: image.port.map(|port| {
              vec![ContainerPort {
                container_port: port as i32,
                protocol: Some("TCP".to_owned()),
                ..Default::default()
              }]
            }),
            resources: Some(ResourceRequirements {
              limits: Some(
                [
                  ("cpu", image.cpu.to_string()),
                  ("memory", image.mem.clone()),
                ]
                .iter()
                .cloned()
                .map(|(k, v)| (k.to_owned(), Quantity(v)))
                .collect(),
              ),
              ..Default::default()
            }),
            ..Default::default()
          })
          .collect(),
        ..Default::default()
      }),
      ..Default::default()
    };
    self.create_pod(pod).await?;
    Ok(())
  }

  pub async fn delete_challenge_env(&self, user_id: i64) -> Result<(), ClusterError> {
    let pod = self
      .get_pods_by_label(&format!("ret.sh.cn/user_id={user_id}"))
      .await?;
    for p in pod {
      self.delete_pod(p.metadata.name.as_ref().unwrap()).await?;
    }
    Ok(())
  }

  pub async fn get_challenge_env(&self, user_id: i64) -> Result<Option<Pod>, ClusterError> {
    let pod = self
      .get_pods_by_label(&format!("ret.sh.cn/user_id={user_id}"))
      .await?;
    Ok(pod.first().cloned())
  }

  pub async fn get_challenge_env_by_team(&self, team_id: i64) -> Result<Vec<Pod>, ClusterError> {
    let pod = self
      .get_pods_by_label(&format!("ret.sh.cn/team_id={team_id}"))
      .await?;
    Ok(pod)
  }

  pub async fn wsrx_link(&self, token: &str, port: u16, ws: WebSocket) -> Result<(), ClusterError> {
    let pod = self
      .get_pods_by_label(&format!("ret.sh.cn/wsrx={token}"))
      .await?;
    let pod = pod
      .first()
      .ok_or(ClusterError::PodNotFound(token.to_owned()))?;
    let client = check_enabled!(self.client)?;
    let api: Api<Pod> = Api::namespaced(client, &with_namespace!(&self.namespace, "wsrx link")?);
    let mut pf = api.portforward(&pod.name().unwrap(), &vec![port]).await?;
    let pfw = pf.take_stream(port);
    if let Some(pfw) = pfw {
      let stream = Framed::new(pfw, wsrx::proxy::MessageCodec::new());
      let ws: wsrx::WrappedWsStream = ws.into();
      wsrx::proxy::proxy_stream(stream, ws).await?;
    }
    Ok(())
  }
}
