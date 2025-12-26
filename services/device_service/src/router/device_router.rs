use crate::{
    AppState,
    database::device::Device,
    router::{
        error::HttpResult,
        prom_service::{get_device_query_timeseries, request_device_query_operations},
        service::{
            request_device_activate, request_device_config, request_device_config_update,
            request_device_status, request_device_token,
        },
    },
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderValue, StatusCode, header},
    response::IntoResponse,
    routing::{get, post, put},
};
use greenhouse_core::{
    device_service_dto::{
        endpoints::{ACTIVATE, CONFIG, STATUS},
        get_device::{DeviceResponseDto, DevicesResponseDto},
        get_timeseries::GetTimeseriesDto,
        operations::OperationsDto,
        post_device::PostDeviceDtoRequest,
        put_device::PutDeviceDtoRequest,
        query::PromQuery,
    },
    smart_device_dto::activation::ActivateRequestDto,
};
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
        .route("/{id}/options", get(get_device_query_operations))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn update_device(
    State(AppState { config: _, pool }): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update): Json<PutDeviceDtoRequest>,
) -> HttpResult<DeviceResponseDto> {
    let mut entry = Device::find_by_id(id, &pool).await?;
    entry.name = update.name.clone();
    entry.description = update.description.clone();
    entry.address = update.address.clone();
    entry.canscript = update.can_script;
    entry.scraping = update.scraping;
    entry.flush(&pool).await?;

    Ok(entry.into())
}

#[axum::debug_handler]
pub(crate) async fn create_device(
    State(AppState { config, pool }): State<AppState>,
    Json(entry): Json<PostDeviceDtoRequest>,
) -> HttpResult<DeviceResponseDto> {
    let mut device = Device::new(
        &entry.name,
        &entry.description,
        &entry.address,
        entry.can_script,
        entry.scraping,
    );
    device.flush(&pool).await?;

    let _ = request_device_activate(
        &entry.address,
        ActivateRequestDto {
            url: config.scripting_api.clone(),
            token: request_device_token(&config.scripting_service).await?.token,
        },
    )
    .await;

    Ok(device.into())
}

#[axum::debug_handler]
pub(crate) async fn get_device(
    State(AppState { config: _, pool }): State<AppState>,
    Path(id): Path<Uuid>,
) -> HttpResult<DeviceResponseDto> {
    Ok(Device::find_by_id(id, &pool).await?.into())
}

#[axum::debug_handler]
pub(crate) async fn get_device_config(
    State(AppState { config: _, pool }): State<AppState>,
    Path(id): Path<Uuid>,
) -> HttpResult<impl IntoResponse> {
    let device = Device::find_by_id(id, &pool).await?;
    let response = request_device_config(&device.address).await?;
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
    State(AppState { config: _, pool }): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> HttpResult<StatusCode> {
    let device = Device::find_by_id(id, &pool).await?;
    let _ = request_device_config_update(&device.address, payload).await?;
    Ok(StatusCode::OK)
}

#[axum::debug_handler]
pub(crate) async fn get_device_status(
    State(AppState { config: _, pool }): State<AppState>,
    Path(id): Path<Uuid>,
) -> HttpResult<impl IntoResponse> {
    let device = Device::find_by_id(id, &pool).await?;
    let response = request_device_status(&device.address).await?;
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
pub(crate) async fn get_devices(
    State(AppState { config: _, pool }): State<AppState>,
) -> HttpResult<DevicesResponseDto> {
    let entries = Device::all(&pool).await?;
    Ok(entries
        .into_iter()
        .map(|entry| entry.into())
        .collect::<Vec<DeviceResponseDto>>()
        .into())
}

#[axum::debug_handler]
pub(crate) async fn activate_device(
    State(AppState { config, pool }): State<AppState>,
    Path(id): Path<Uuid>,
) -> HttpResult<StatusCode> {
    let device = Device::find_by_id(id, &pool).await?;
    let _ = request_device_activate(
        &device.address,
        ActivateRequestDto {
            url: config.scripting_api.clone(),
            token: request_device_token(&config.scripting_service).await?.token,
        },
    )
    .await?;
    Ok(StatusCode::OK)
}

#[axum::debug_handler]
pub(crate) async fn get_device_timeseries(
    State(AppState { config, pool: _ }): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<PromQuery>,
) -> HttpResult<GetTimeseriesDto> {
    Ok(get_device_query_timeseries(&config.prometheus_url, &id.to_string(), query).await?)
}

async fn get_device_query_operations(
    State(AppState { config, pool: _ }): State<AppState>,
    Path(id): Path<Uuid>,
) -> HttpResult<OperationsDto> {
    Ok(request_device_query_operations(&config.prometheus_url, &id.to_string()).await?)
}
