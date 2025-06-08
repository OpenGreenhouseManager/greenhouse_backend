use super::error::Result;
use crate::{AppState, database::device::Device, router::service::request_device_config};
use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post, put},
};
use greenhouse_core::device_service_dto::{
    endpoints::CONFIG, get_device::DeviceResponseDto, post_device::PostDeviceDtoRequest,
    put_device::PutDeviceDtoRequest,
};
use uuid::Uuid;

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_device))
        .route("/", get(get_devices))
        .route("/{id}", put(update_device))
        .route("/{id}", get(get_device))
        .route(&format!("/{{id}}/{CONFIG}"), get(get_device_config))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn update_device(
    State(AppState { config: _, pool }): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update): Json<PutDeviceDtoRequest>,
) -> Result<impl IntoResponse> {
    let mut entry = Device::find_by_id(id, &pool).await?;
    entry.name = update.name.clone();
    entry.description = update.description.clone();

    entry.address = update.address.clone();
    entry.flush(&pool).await?;
    let response: DeviceResponseDto = entry.into();
    Ok(Json(response))
}

#[axum::debug_handler]
pub(crate) async fn create_device(
    State(AppState { config: _, pool }): State<AppState>,
    Json(entry): Json<PostDeviceDtoRequest>,
) -> Result<impl IntoResponse> {
    let mut entry = Device::new(
        &entry.name,
        &entry.description,
        &entry.address,
        entry.can_script,
    );
    entry.flush(&pool).await?;
    let response: DeviceResponseDto = entry.into();
    Ok(Json(response))
}

#[axum::debug_handler]
pub(crate) async fn get_device(
    State(AppState { config: _, pool }): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let entry = Device::find_by_id(id, &pool).await?;
    let response: DeviceResponseDto = entry.into();
    Ok(Json(response))
}

#[axum::debug_handler]
pub(crate) async fn get_device_config(
    State(AppState { config: _, pool }): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let device = Device::find_by_id(id, &pool).await?;
    let response = request_device_config(&device.address).await?;
    Ok(response)
}

#[axum::debug_handler]
pub(crate) async fn get_devices(
    State(AppState { config: _, pool }): State<AppState>,
) -> Result<impl IntoResponse> {
    let entries = Device::all(&pool).await?;
    let response: Vec<DeviceResponseDto> = entries.into_iter().map(|entry| entry.into()).collect();
    Ok(Json(response))
}
