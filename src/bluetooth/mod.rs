use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_streams::StreamBodyAs;
use futures::StreamExt as _;
use serde::Serialize;

use crate::error::{AppOk, AppResult};

#[derive(Debug, Serialize)]
pub struct DeviceInfo {
    pub address: bluer::Address,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub rssi: Option<i16>,
    pub is_connected: bool,
    pub is_paired: bool,
    pub is_trusted: bool,
    pub is_blocked: bool,
}

pub async fn router() -> anyhow::Result<Router> {
    Ok(Router::new()
        .route("/adapters", get(adapters))
        .route("/adapters/{name}/events", get(events))
        .route("/adapters/{name}/discover", post(discover))
        .route("/adapters/{name}/devices", get(devices))
        .route("/adapters/{name}/devices/{addr}/pair", post(device_pair))
        .route(
            "/adapters/{name}/devices/{addr}/unpair",
            post(device_unpair),
        )
        .route(
            "/adapters/{name}/devices/{addr}/connect",
            post(device_connect),
        )
        .route(
            "/adapters/{name}/devices/{addr}/disconnect",
            post(device_disconnect),
        )
        .route("/adapters/{name}/devices/{addr}/trust", post(device_trust))
        .route(
            "/adapters/{name}/devices/{addr}/untrust",
            post(device_untrust),
        )
        .route("/adapters/{name}/devices/{addr}/block", post(device_block))
        .route(
            "/adapters/{name}/devices/{addr}/unblock",
            post(device_unblock),
        )
        .with_state(Arc::new(bluer::Session::new().await?)))
}

async fn adapters(State(session): State<Arc<bluer::Session>>) -> AppResult<Json<Vec<String>>> {
    Ok(Json(session.adapter_names().await?))
}

async fn devices(
    State(session): State<Arc<bluer::Session>>,
    Path(name): Path<String>,
) -> AppResult<Json<Vec<DeviceInfo>>> {
    let adapter = session.adapter(&name)?;
    let devices = futures::stream::iter(adapter.device_addresses().await?)
        .map(|addr| {
            let adapter = adapter.clone();
            async move {
                let device = adapter.device(addr)?;
                Ok(DeviceInfo {
                    address: addr,
                    name: device.name().await?,
                    icon: device.icon().await?,
                    rssi: device.rssi().await?,
                    is_connected: device.is_connected().await?,
                    is_paired: device.is_paired().await?,
                    is_trusted: device.is_trusted().await?,
                    is_blocked: device.is_blocked().await?,
                })
            }
        })
        .buffer_unordered(4)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<AppResult<Vec<_>>>()?;

    Ok(Json(devices))
}

async fn discover(
    State(session): State<Arc<bluer::Session>>,
    Path(name): Path<String>,
) -> AppResult<impl IntoResponse> {
    let adapter = session.adapter(&name)?;
    Ok(StreamBodyAs::json_nl(adapter.discover_devices().await?))
}

async fn events(
    State(session): State<Arc<bluer::Session>>,
    Path(name): Path<String>,
) -> AppResult<impl IntoResponse> {
    let adapter = session.adapter(&name)?;
    Ok(StreamBodyAs::json_nl(adapter.events().await?))
}

async fn device_pair(
    State(session): State<Arc<bluer::Session>>,
    Path((name, addr)): Path<(String, bluer::Address)>,
) -> AppResult<AppOk> {
    let adapter = session.adapter(&name)?;
    adapter.device(addr)?.pair().await?;
    Ok(AppOk)
}

async fn device_unpair(
    State(session): State<Arc<bluer::Session>>,
    Path((name, addr)): Path<(String, bluer::Address)>,
) -> AppResult<AppOk> {
    let adapter = session.adapter(&name)?;
    adapter.remove_device(addr).await?;
    Ok(AppOk)
}

async fn device_connect(
    State(session): State<Arc<bluer::Session>>,
    Path((name, addr)): Path<(String, bluer::Address)>,
) -> AppResult<AppOk> {
    let adapter = session.adapter(&name)?;
    dbg!(adapter.device(addr)?.connect().await)?;
    Ok(AppOk)
}

async fn device_disconnect(
    State(session): State<Arc<bluer::Session>>,
    Path((name, addr)): Path<(String, bluer::Address)>,
) -> AppResult<AppOk> {
    let adapter = session.adapter(&name)?;
    adapter.device(addr)?.disconnect().await?;
    Ok(AppOk)
}

async fn device_trust(
    State(session): State<Arc<bluer::Session>>,
    Path((name, addr)): Path<(String, bluer::Address)>,
) -> AppResult<AppOk> {
    let adapter = session.adapter(&name)?;
    adapter.device(addr)?.set_trusted(true).await?;
    Ok(AppOk)
}

async fn device_untrust(
    State(session): State<Arc<bluer::Session>>,
    Path((name, addr)): Path<(String, bluer::Address)>,
) -> AppResult<AppOk> {
    let adapter = session.adapter(&name)?;
    adapter.device(addr)?.set_trusted(false).await?;
    Ok(AppOk)
}

async fn device_block(
    State(session): State<Arc<bluer::Session>>,
    Path((name, addr)): Path<(String, bluer::Address)>,
) -> AppResult<AppOk> {
    let adapter = session.adapter(&name)?;
    adapter.device(addr)?.set_blocked(true).await?;
    Ok(AppOk)
}

async fn device_unblock(
    State(session): State<Arc<bluer::Session>>,
    Path((name, addr)): Path<(String, bluer::Address)>,
) -> AppResult<AppOk> {
    let adapter = session.adapter(&name)?;
    adapter.device(addr)?.set_blocked(false).await?;
    Ok(AppOk)
}
