use crate::{
    AppState,
    database::device::Device,
    router::{
        error::HttpResult,
        service::{
            request_device_activate, request_device_config, request_device_status,
            request_device_token,
        },
    },
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::IntoResponse,
    routing::{get, post, put},
};
use greenhouse_core::{
    device_service_dto::{
        endpoints::{ACTIVATE, CONFIG, STATUS},
        get_device::DeviceResponseDto,
        post_device::PostDeviceDtoRequest,
        put_device::PutDeviceDtoRequest,
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
        .route(&format!("/{{id}}/{CONFIG}"), get(get_device_config))
        .route(&format!("/{{id}}/{STATUS}"), get(get_device_status))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn update_device(
    State(AppState { config: _, pool }): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update): Json<PutDeviceDtoRequest>,
) -> HttpResult<impl IntoResponse> {
    let mut entry = Device::find_by_id(id, &pool).await?;
    entry.name = update.name.clone();
    entry.description = update.description.clone();
    entry.address = update.address.clone();
    entry.canscript = update.can_script;
    entry.flush(&pool).await?;

    let response: DeviceResponseDto = entry.into();
    Ok(Json(response))
}

#[axum::debug_handler]
pub(crate) async fn create_device(
    State(AppState { config, pool }): State<AppState>,
    Json(entry): Json<PostDeviceDtoRequest>,
) -> HttpResult<impl IntoResponse> {
    let mut device = Device::new(
        &entry.name,
        &entry.description,
        &entry.address,
        entry.can_script,
    );
    device.flush(&pool).await?;

    let _ = request_device_activate(
        &entry.address,
        ActivateRequestDto {
            url: config.scripting_service.clone(),
            token: request_device_token(&config.scripting_service).await?.token,
        },
    )
    .await;

    let response: DeviceResponseDto = device.into();
    Ok(Json(response))
}

#[axum::debug_handler]
pub(crate) async fn get_device(
    State(AppState { config: _, pool }): State<AppState>,
    Path(id): Path<Uuid>,
) -> HttpResult<impl IntoResponse> {
    let entry = Device::find_by_id(id, &pool).await?;
    let response: DeviceResponseDto = entry.into();
    Ok(Json(response))
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
) -> HttpResult<impl IntoResponse> {
    let entries = Device::all(&pool).await?;
    let response: Vec<DeviceResponseDto> = entries.into_iter().map(|entry| entry.into()).collect();
    Ok(Json(response))
}

#[axum::debug_handler]
pub(crate) async fn activate_device(
    State(AppState { config, pool }): State<AppState>,
    Path(id): Path<Uuid>,
) -> HttpResult<impl IntoResponse> {
    let device = Device::find_by_id(id, &pool).await?;
    let _ = request_device_activate(
        &device.address,
        ActivateRequestDto {
            url: config.scripting_service.clone(),
            token: request_device_token(&config.scripting_service).await?.token,
        },
    )
    .await?;
    Ok(StatusCode::OK.into_response())
}
