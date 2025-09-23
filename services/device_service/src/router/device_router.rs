use super::error::Error;

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
    extract::{Path, Query, State},
    http::{HeaderValue, StatusCode, header},
    response::IntoResponse,
    routing::{get, post, put},
};
use greenhouse_core::{
    device_service_dto::{
        endpoints::{ACTIVATE, CONFIG, STATUS},
        get_device::DeviceResponseDto,
        get_timeseries::{TimeseriesDto, Type},
        post_device::PostDeviceDtoRequest,
        put_device::PutDeviceDtoRequest,
        query::PromQuery,
    },
    smart_device_dto::activation::ActivateRequestDto,
};
use reqwest::Client;
use uuid::Uuid;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct PrometheusResponse {
    pub status: String,
    pub data: PrometheusData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PrometheusData {
    #[serde(rename = "resultType")]
    pub result_type: String,
    pub result: Vec<PrometheusResult>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PrometheusResult {
    pub metric: HashMap<String, String>,
    pub values: Vec<(u64, String)>, // (timestamp, value as string)
}

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_device))
        .route("/", get(get_devices))
        .route("/{id}", put(update_device))
        .route("/{id}", get(get_device))
        .route(&format!("/{{id}}/{ACTIVATE}"), put(activate_device))
        .route(&format!("/{{id}}/{CONFIG}"), get(get_device_config))
        .route(&format!("/{{id}}/{STATUS}"), get(get_device_status))
        .route("/{id}/timeseries", get(get_device_timeseries))
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
    entry.scraping = update.scraping;
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
            url: config.scripting_api.clone(),
            token: request_device_token(&config.scripting_service).await?.token,
        },
    )
    .await?;
    Ok(StatusCode::OK.into_response())
}

#[axum::debug_handler]
pub(crate) async fn get_device_timeseries(
    State(AppState { config, pool: _ }): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<PromQuery>,
) -> HttpResult<impl IntoResponse> {
    let client = Client::new();

    // Example metric name: scrape_service_duration_<uuid>_periodic_alert_4
    let id = id.to_string().replace("-", "_");
    let name = if let Some(sub_property) = query.sub_property {
        format!("scrape_service_duration_{id}_{sub_property}")
    } else {
        format!("scrape_service_duration_{id}")
    };

    // Convert chrono DateTime<Utc> to unix seconds
    let start = query.start.timestamp();
    let end = query.end.timestamp();

    // Step (you could make this configurable too)
    let step = query.step.unwrap_or("15s".to_string());

    // Build the URL
    let url = format!(
        "{}/query_range",
        config.prometheus_url.trim_end_matches('/')
    );

    tracing::info!("Prometheus URL: {}", url);
    tracing::info!("Prometheus query: {}", name);
    tracing::info!("Prometheus start: {}", start);
    tracing::info!("Prometheus end: {}", end);
    tracing::info!("Prometheus step: {}", step);

    // Build the request with query params
    let resp = client
        .get(url)
        .query(&[
            ("query", &name),
            ("start", &start.to_string()),
            ("end", &end.to_string()),
            ("step", &step),
        ])
        .send()
        .await
        .map_err(Error::Prometheus)?
        .json::<PrometheusResponse>()
        .await
        .map_err(Error::PrometheusJson)?;

    match resp.data.result[0]
        .metric
        .get("type")
        .unwrap_or(&String::from("unknown"))
        .as_str()
    {
        "array" => {
            tracing::error!("Array type not implemented: {}", resp.data.result_type);
            Err(Error::PrometheusNotImplemented.into())
        }
        "number" => {
            let timeseries: Vec<TimeseriesDto> = resp.data.result[0]
                .values
                .iter()
                .map(|result| TimeseriesDto {
                    timestamp: result.0,
                    value: Type::Number(result.1.parse::<f64>().unwrap()),
                })
                .collect();
            Ok(Json(timeseries))
        }
        "boolean" => {
            let timeseries: Vec<TimeseriesDto> = resp.data.result[0]
                .values
                .iter()
                .map(|result| TimeseriesDto {
                    timestamp: result.0,
                    value: Type::Boolean(result.1.parse::<bool>().unwrap()),
                })
                .collect();
            Ok(Json(timeseries))
        }
        "object" => {
            tracing::error!("Object type not implemented: {}", resp.data.result_type);
            Err(Error::PrometheusNotImplemented.into())
        }
        _ => {
            tracing::error!("Prometheus invalid result type: {}", resp.data.result_type);
            Err(Error::PrometheusInvalidResultType.into())
        }
    }
}
