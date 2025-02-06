use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
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
}

pub async fn router() -> anyhow::Result<Router> {
    Ok(Router::new()
        .route("/adapters", get(adapters))
        .route("/adapters/{name}/events", get(events))
        .route("/adapters/{name}/discover", get(discover))
        .route("/adapters/{name}/devices", get(devices))
        .route("/adapters/{name}/devices/{addr}/pair", get(device_pair))
        .route(
            "/adapters/{name}/devices/{addr}/connect",
            get(device_connect),
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

async fn device_connect(
    State(session): State<Arc<bluer::Session>>,
    Path((name, addr)): Path<(String, bluer::Address)>,
) -> AppResult<AppOk> {
    let adapter = session.adapter(&name)?;
    dbg!(adapter.device(addr)?.connect().await)?;
    Ok(AppOk)
}
