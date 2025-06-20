use super::error::Result;
use crate::{AppState, device::service};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderValue,
    response::IntoResponse,
    routing::{get, post, put},
};
use greenhouse_core::device_service_dto::{
    endpoints::{CONFIG, STATUS},
    post_device::PostDeviceDtoRequest,
    put_device::PutDeviceDtoRequest,
};
use reqwest::{StatusCode, header};
use uuid::Uuid;

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_device))
        .route("/", get(get_devices))
        .route("/{id}", put(update_device))
        .route("/{id}", get(get_device))
        .route(&format!("/{{id}}/{CONFIG}"), get(get_device_config))
        .route(&format!("/{{id}}/{STATUS}"), get(get_device_status))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn create_device(
    State(AppState { config }): State<AppState>,
    Json(entry): Json<PostDeviceDtoRequest>,
) -> Result<impl IntoResponse> {
    let device =
        service::create_device(&config.service_addresses.data_storage_service, entry).await?;
    Ok(Json(device))
}

#[axum::debug_handler]
pub(crate) async fn get_devices(
    State(AppState { config }): State<AppState>,
) -> Result<impl IntoResponse> {
    let devices = service::get_devices(&config.service_addresses.data_storage_service).await?;
    Ok(Json(devices))
}

#[axum::debug_handler]
pub(crate) async fn update_device(
    State(AppState { config }): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update): Json<PutDeviceDtoRequest>,
) -> Result<impl IntoResponse> {
    let device =
        service::update_device(&config.service_addresses.data_storage_service, id, update).await?;
    Ok(Json(device))
}

#[axum::debug_handler]
pub(crate) async fn get_device(
    State(AppState { config }): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let device = service::get_device(&config.service_addresses.data_storage_service, id).await?;
    Ok(Json(device))
}

#[axum::debug_handler]
pub(crate) async fn get_device_config(
    State(AppState { config }): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let response =
        service::get_device_config(&config.service_addresses.data_storage_service, id).await?;
    Ok((
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )],
        response,
    ))
}

#[axum::debug_handler]
pub(crate) async fn get_device_status(
    State(AppState { config }): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let response =
        service::get_device_status(&config.service_addresses.data_storage_service, id).await?;
    Ok((
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )],
        response,
    ))
}
