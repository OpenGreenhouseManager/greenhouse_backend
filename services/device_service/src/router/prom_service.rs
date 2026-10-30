use super::error::{Error, Result};
use futures::{StreamExt, stream};
use greenhouse_core::{
    device_service_dto::{
        batch_timeseries::{
            BatchTimeseriesRequestDto, BatchTimeseriesResponseDto, BatchTimeseriesResultDto,
        },
        get_timeseries::{GetTimeseriesDto, Measurement, TimeseriesDto, Type},
        operations::OperationsDto,
        query::PromQuery,
    },
    http_error::HttpErrorMapping,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct PrometheusTimeseriesResponse {
    pub status: String,
    pub data: PrometheusTimeseriesData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PrometheusTimeseriesData {
    #[serde(rename = "resultType")]
    pub result_type: String,
    pub result: Vec<PrometheusTimeseriesResult>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PrometheusTimeseriesResult {
    pub metric: HashMap<String, String>,
    pub values: Vec<(u64, String)>, // (timestamp, value as string)
}

#[derive(Deserialize, Serialize)]
pub struct PrometheusMetricNameResponse {
    pub status: String,
    pub data: Vec<MetricSeries>,
}

#[derive(Deserialize, Serialize)]
pub struct MetricSeries {
    #[serde(rename = "__name__")]
    pub name: String,

    pub instance: String,
    pub job: String,

    /// Present only when type == "string"
    #[serde(default)]
    pub string_value: Option<String>,

    #[serde(rename = "type")]
    pub metric_type: String,
}

/// Upper bound for queries in a single batch request.
pub(crate) const MAX_BATCH_QUERIES: usize = 50;
/// How many Prometheus requests a batch runs concurrently.
const BATCH_CONCURRENCY: usize = 8;

pub(crate) async fn get_device_query_timeseries(
    prometheus_url: &str,
    device_id: Uuid,
    query: PromQuery,
) -> Result<GetTimeseriesDto> {
    Ok(
        fetch_timeseries(&Client::new(), prometheus_url, device_id, &query)
            .await?
            .into(),
    )
}

pub(crate) async fn get_batch_query_timeseries(
    prometheus_url: &str,
    request: BatchTimeseriesRequestDto,
) -> Result<BatchTimeseriesResponseDto> {
    if request.queries.len() > MAX_BATCH_QUERIES {
        return Err(Error::BatchTooLarge);
    }

    let client = &Client::new();

    let results = stream::iter(request.queries)
        .map(|query| async move {
            let result =
                fetch_timeseries(client, prometheus_url, query.device_id, &query.query).await;

            let (timeseries, error) = match result {
                Ok(timeseries) => (Some(timeseries), None),
                Err(e) => {
                    tracing::warn!("Batch timeseries query {query:?} failed: {e}");
                    (None, Some(e.to_error_message()))
                }
            };

            BatchTimeseriesResultDto {
                query,
                timeseries,
                error,
            }
        })
        // `buffered` keeps the request order while bounding concurrency
        .buffered(BATCH_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;

    Ok(results.into())
}

async fn fetch_timeseries(
    client: &Client,
    prometheus_url: &str,
    device_id: Uuid,
    query: &PromQuery,
) -> Result<Vec<TimeseriesDto>> {
    // Example metric name: scrape_service_duration_<uuid>_periodic_alert_4
    let id = device_id.to_string().replace("-", "_");
    let name = if let Some(sub_property) = &query.sub_property {
        format!("scrape_service_duration_{id}_{sub_property}")
    } else {
        format!("scrape_service_duration_{id}")
    };

    let url = format!("{}/query_range", prometheus_url.trim_end_matches('/'));

    tracing::debug!("Prometheus query_range {url}: query={name} {query:?}");

    let resp = client
        .get(url)
        .query(&[
            ("query", name.as_str()),
            ("start", &query.start.timestamp().to_string()),
            ("end", &query.end.timestamp().to_string()),
            ("step", &query.step.to_string()),
        ])
        .send()
        .await
        .map_err(Error::Prometheus)?
        .json::<PrometheusTimeseriesResponse>()
        .await
        .map_err(Error::PrometheusJson)?;

    // No samples in the requested window
    let Some(series) = resp.data.result.first() else {
        return Ok(Vec::new());
    };

    let parse_f64 = |value: &str| {
        value
            .parse::<f64>()
            .map_err(|_| Error::PrometheusInvalidValue)
    };

    match series
        .metric
        .get("type")
        .map(String::as_str)
        .unwrap_or("unknown")
    {
        "number" => series
            .values
            .iter()
            .map(|(timestamp, value)| {
                Ok(TimeseriesDto {
                    timestamp: *timestamp,
                    value: Type::Number(parse_f64(value)?),
                })
            })
            .collect(),
        // Booleans are stored as 1.0 / 0.0 gauges by the scrape service
        "boolean" => series
            .values
            .iter()
            .map(|(timestamp, value)| {
                Ok(TimeseriesDto {
                    timestamp: *timestamp,
                    value: Type::Boolean(parse_f64(value)? != 0.0),
                })
            })
            .collect(),
        "object" => {
            tracing::error!("Object type not implemented: {}", resp.data.result_type);
            Err(Error::PrometheusNotImplemented)
        }
        "measurement" => {
            let unit = series
                .metric
                .get("unit")
                .cloned()
                .unwrap_or("unknown".to_string());

            series
                .values
                .iter()
                .map(|(timestamp, value)| {
                    Ok(TimeseriesDto {
                        timestamp: *timestamp,
                        value: Type::Measurement(Measurement {
                            value: parse_f64(value)?,
                            unit: unit.clone(),
                        }),
                    })
                })
                .collect()
        }
        _ => {
            tracing::error!("Prometheus invalid result type: {}", resp.data.result_type);
            Err(Error::PrometheusInvalidResultType)
        }
    }
}

pub(crate) async fn request_device_query_operations(
    prometheus_url: &str,
    id: &str,
) -> Result<OperationsDto> {
    let client = Client::new();
    let id = id.to_string().replace("-", "_");
    let url_prefix = "{__name__=~'";
    let url_suffix = ".*'}";
    let resp = client
        .get(format!("{}/series", prometheus_url.trim_end_matches('/')))
        .query(&[(
            "match[]",
            &format!("{url_prefix}scrape_service_duration_{id}_{url_suffix}"),
        )])
        .send()
        .await
        .map_err(Error::Prometheus)?
        .json::<PrometheusMetricNameResponse>()
        .await
        .map_err(Error::PrometheusJson)?;

    Ok(resp
        .data
        .iter()
        .filter_map(|series| {
            series
                .name
                .strip_prefix(&format!("scrape_service_duration_{id}_"))
                .map(|s| s.to_string())
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<String>>()
        .into())
}
