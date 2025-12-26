use crate::{AppState, device::service, helper::error::HttpResult};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderValue,
    response::IntoResponse,
    routing::{get, post, put},
};
use greenhouse_core::device_service_dto::{
    endpoints::{ACTIVATE, CONFIG, STATUS},
    get_device::{DeviceResponseDto, DevicesResponseDto},
    get_timeseries::GetTimeseriesDto,
    operations::OperationsDto,
    post_device::PostDeviceDtoRequest,
    put_device::PutDeviceDtoRequest,
    query::PromQuery,
};
use reqwest::{StatusCode, header};
use uuid::Uuid;

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_device))
        .route("/", get(get_devices))
        .route("/{id}", put(update_device))
        .route("/{id}", get(get_device))
        .route(&format!("/{{id}}/{ACTIVATE}"), put(activate_device))
        .route(&format!("/{{id}}/{CONFIG}"), put(update_device_config))
        .route(&format!("/{{id}}/{CONFIG}"), get(get_device_config))
        .route(&format!("/{{id}}/{STATUS}"), get(get_device_status))
        .route("/{id}/timeseries", get(get_device_timeseries))
        .route("/{id}/options", get(get_device_operations))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn create_device(
    State(AppState { config }): State<AppState>,
    Json(entry): Json<PostDeviceDtoRequest>,
) -> HttpResult<DeviceResponseDto> {
    Ok(service::create_device(&config.service_addresses.device_service, entry).await?)
}

#[axum::debug_handler]
pub(crate) async fn get_devices(
    State(AppState { config }): State<AppState>,
) -> HttpResult<DevicesResponseDto> {
    Ok(service::get_devices(&config.service_addresses.device_service).await?)
}

#[axum::debug_handler]
pub(crate) async fn update_device(
    State(AppState { config }): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update): Json<PutDeviceDtoRequest>,
) -> HttpResult<DeviceResponseDto> {
    Ok(service::update_device(&config.service_addresses.device_service, id, update).await?)
}

#[axum::debug_handler]
pub(crate) async fn get_device(
    State(AppState { config }): State<AppState>,
    Path(id): Path<Uuid>,
) -> HttpResult<DeviceResponseDto> {
    Ok(service::get_device(&config.service_addresses.device_service, id).await?)
}

#[axum::debug_handler]
pub(crate) async fn get_device_config(
    State(AppState { config }): State<AppState>,
    Path(id): Path<Uuid>,
) -> HttpResult<impl IntoResponse> {
    let response = service::get_device_config(&config.service_addresses.device_service, id).await?;
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
pub(crate) async fn update_device_config(
    State(AppState { config }): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> HttpResult<StatusCode> {
    service::update_device_config(&config.service_addresses.device_service, id, body).await?;
    Ok(StatusCode::OK)
}

#[axum::debug_handler]
pub(crate) async fn get_device_status(
    State(AppState { config }): State<AppState>,
    Path(id): Path<Uuid>,
) -> HttpResult<impl IntoResponse> {
    let response = service::get_device_status(&config.service_addresses.device_service, id).await?;
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
pub(crate) async fn activate_device(
    State(AppState { config }): State<AppState>,
    Path(id): Path<Uuid>,
) -> HttpResult<StatusCode> {
    service::activate_device(&config.service_addresses.device_service, id).await?;
    Ok(StatusCode::OK)
}

#[axum::debug_handler]
pub(crate) async fn get_device_timeseries(
    State(AppState { config }): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<PromQuery>,
) -> HttpResult<GetTimeseriesDto> {
    Ok(service::get_device_timeseries(&config.service_addresses.device_service, id, query).await?)
}

pub(crate) async fn get_device_operations(
    State(AppState { config }): State<AppState>,
    Path(id): Path<Uuid>,
) -> HttpResult<OperationsDto> {
    Ok(service::get_device_operations(&config.service_addresses.device_service, id).await?)
}
