use axum::{
    extract::{Extension, State},
    http::StatusCode,
    middleware::{self, from_fn_with_state},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use git_version::git_version;
use k8s_openapi::api::core::v1::{ConfigMap, Node};
use kube::{api::ListParams, core::ObjectList, Api};
use rustc_version::version;
use sea_orm::DatabaseConnection;
use serde::Serialize;
use systemstat::{CPULoad, Filesystem, Memory, Platform, Swap, System};
use tracing::error;

use crate::{
    cache,
    cache::manager::RedisPool,
    controller::{
        layer::auth::{self, init_token_or_permission_required},
        GlobalState,
    },
    entity::{
        config::{self, Model as ConfigModel},
        user::Permission,
    },
};

pub fn router(state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route(
            "/config",
            get(get_platform_config)
                .post(set_platform_config)
                .head(test_platform_init_token),
        )
        .route_layer(from_fn_with_state(
            state.clone(),
            init_token_or_permission_required,
        ))
        .nest(
            "/",
            Router::new()
                .route("/cluster", get(get_cluster_stat))
                .route("/stat", get(get_platform_stat))
                .route_layer(middleware::from_fn(auth::permission_required_all!(
                    Permission::Devops
                ))),
        )
        .route("/version", get(get_platform_version))
        .route("/", get(get_platform_info))
}

async fn get_platform_info(
    platform_info: Option<Extension<ConfigModel>>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    if let Some(Extension(platform_info)) = platform_info {
        Ok(Json(platform_info.platform))
    } else {
        error!("platform info not found");
        Err((StatusCode::NOT_FOUND, "platform info not found"))
    }
}

async fn get_platform_version() -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    Ok(Json(format!(
        "{}-{}-{}",
        env!("CARGO_PKG_VERSION"),
        git_version!(
            args = ["--abbrev=8", "--always", "--dirty=*"],
            fallback = "unknown"
        )
        .to_uppercase(),
        version().unwrap().to_string()
    )))
}

#[derive(Serialize)]
struct MemoryOriginal {
    pub total: u64,
    pub free: u64,
}

impl From<Memory> for MemoryOriginal {
    fn from(memory: Memory) -> Self {
        Self {
            total: memory.total.as_u64(),
            free: memory.free.as_u64(),
        }
    }
}

impl From<Swap> for MemoryOriginal {
    fn from(swap: Swap) -> Self {
        Self {
            total: swap.total.as_u64(),
            free: swap.free.as_u64(),
        }
    }
}

#[derive(Serialize)]
struct FilesystemOriginal {
    /// Free bytes in filesystem
    pub free: u64,
    /// Free bytes available to non-superuser
    pub avail: u64,
    /// Total bytes in filesystem
    pub total: u64,
    pub fs_type: String,
    pub fs_mounted_from: String,
    pub fs_mounted_on: String,
}

impl From<Filesystem> for FilesystemOriginal {
    fn from(filesystem: Filesystem) -> Self {
        Self {
            free: filesystem.free.as_u64(),
            avail: filesystem.avail.as_u64(),
            total: filesystem.total.as_u64(),
            fs_type: filesystem.fs_type,
            fs_mounted_from: filesystem.fs_mounted_from,
            fs_mounted_on: filesystem.fs_mounted_on,
        }
    }
}

#[derive(Serialize)]
struct PlatformStat {
    pub cpu: Vec<CPULoad>,
    pub memory: MemoryOriginal,
    pub swap: MemoryOriginal,
    pub disks: Vec<FilesystemOriginal>,
    pub uptime: u64,
}

async fn get_platform_stat() -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let sys = System::new();
    let cpu_load: systemstat::DelayedMeasurement<Vec<CPULoad>> = sys.cpu_load().map_err(|err| {
        error!("failed to get cpu load error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to get cpu load error",
        )
    })?;
    let cal_start_time = tokio::time::Instant::now();
    let cal_end_time = cal_start_time + tokio::time::Duration::from_millis(500);

    let (memory, swap) = sys.memory_and_swap().map_err(|err| {
        error!("failed to get memory and swap error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to get memory and swap error",
        )
    })?;
    let disks = sys.mounts().map_err(|err| {
        error!("failed to get disk error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to get disk error",
        )
    })?;
    let disks: Vec<FilesystemOriginal> = disks
        .into_iter()
        .map(|filesystem| filesystem.into())
        .collect();
    let uptime = sys
        .uptime()
        .map_err(|err| {
            error!("failed to get uptime error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get uptime error",
            )
        })?
        .as_secs();

    tokio::time::sleep_until(cal_end_time).await;
    let cpu = cpu_load.done().map_err(|err| {
        error!("failed to get cpu load error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to get cpu load error",
        )
    })?;
    Ok(Json(PlatformStat {
        cpu,
        memory: memory.into(),
        swap: swap.into(),
        disks,
        uptime,
    }))
}

async fn get_platform_config(
    platform_info: Option<Extension<ConfigModel>>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    if let Some(Extension(platform_info)) = platform_info {
        Ok(Json(platform_info))
    } else {
        error!("platform config not found");
        Err((StatusCode::NOT_FOUND, "platform config not found"))
    }
}

async fn set_platform_config(
    State(ref db): State<DatabaseConnection>, State(ref mut cache): State<RedisPool>,
    Json(new_model): Json<ConfigModel>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    config::update_config(db, new_model).await.map_err(|err| {
        error!("failed to update platform error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to update platform error [DbErr]",
        )
    })?;
    cache::platform::Platform::refresh_cache(cache, db)
        .await
        .map_err(|err| {
            error!("failed to update platform error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to update platform error [CacheErr]",
            )
        })?;
    Ok(StatusCode::OK)
}

async fn test_platform_init_token() -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    Ok(StatusCode::OK)
}

#[derive(Serialize)]
struct ClusterInfo {
    pub version: k8s_openapi::apimachinery::pkg::version::Info,
    pub default_namespace: String,
    pub nodes: ObjectList<Node>,
    pub configs: ObjectList<ConfigMap>,
}

async fn get_cluster_stat(
    State(cluster): State<kube::Client>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let version = cluster.apiserver_version().await.map_err(|err| {
        error!("failed to get cluster stat error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to get cluster stat error",
        )
    })?;
    let default_namespace = String::from(cluster.default_namespace());
    let cms = Api::<ConfigMap>::all(cluster.clone());
    let configs = cms.list(&ListParams::default()).await.map_err(|err| {
        error!("failed to get cluster stat error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to get cluster stat error",
        )
    })?;
    let api: Api<Node> = Api::all(cluster);
    let nodes = api.list(&ListParams::default()).await.map_err(|err| {
        error!("failed to get cluster stat error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to get cluster stat error",
        )
    })?;
    Ok(Json(ClusterInfo {
        version,
        default_namespace,
        nodes,
        configs,
    }))
}

#[cfg(test)]
mod tests {
    use std::thread;

    use systemstat::{Duration, Platform, System};

    #[test]
    fn test_systemstat() {
        let sys = System::new();

        match sys.mounts() {
            Ok(mounts) => {
                println!("\nMounts:");
                for mount in mounts.iter() {
                    println!("{:?}", mount);
                }
            }
            Err(x) => println!("\nMounts: error: {}", x),
        }

        match sys.memory() {
            Ok(mem) => println!("\nMemory: {:?}", mem),
            Err(x) => println!("\nMemory: error: {}", x),
        }

        match sys.swap() {
            Ok(swap) => println!("\nSwap: {:?}", swap),
            Err(x) => println!("\nSwap: error: {}", x),
        }

        match sys.load_average() {
            Ok(loadavg) => println!(
                "\nLoad average: {} {} {}",
                loadavg.one, loadavg.five, loadavg.fifteen
            ),
            Err(x) => println!("\nLoad average: error: {}", x),
        }

        match sys.uptime() {
            Ok(uptime) => println!("\nUptime: {:?}", uptime),
            Err(x) => println!("\nUptime: error: {}", x),
        }

        match sys.boot_time() {
            Ok(boot_time) => println!("\nBoot time: {}", boot_time),
            Err(x) => println!("\nBoot time: error: {}", x),
        }

        match sys.cpu_load_aggregate() {
            Ok(cpu) => {
                println!("\nMeasuring CPU load...");
                thread::sleep(Duration::from_secs(1));
                let cpu = cpu.done().unwrap();
                println!(
                    "CPU load: {}% user, {}% nice, {}% system, {}% intr, {}% idle ",
                    cpu.user * 100.0,
                    cpu.nice * 100.0,
                    cpu.system * 100.0,
                    cpu.interrupt * 100.0,
                    cpu.idle * 100.0
                );
            }
            Err(x) => println!("\nCPU load: error: {}", x),
        }

        match sys.cpu_temp() {
            Ok(cpu_temp) => println!("\nCPU temp: {:?}", cpu_temp),
            Err(x) => println!("\nCPU temp: {}", x),
        }

        match sys.socket_stats() {
            Ok(stats) => println!("\nSystem socket statistics: {:?}", stats),
            Err(x) => println!("\nSystem socket statistics: error: {}", x),
        }
    }
}
