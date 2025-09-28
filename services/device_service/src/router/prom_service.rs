use super::error::{Error, Result};
use greenhouse_core::device_service_dto::{
    get_timeseries::{TimeseriesDto, Type},
    query::PromQuery,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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

pub(crate) async fn get_device_query_timeseries(
    prometheus_url: &str,
    id: &str,
    query: PromQuery,
) -> Result<Vec<TimeseriesDto>> {
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
    let url = format!("{}/query_range", prometheus_url.trim_end_matches('/'));

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
        .json::<PrometheusTimeseriesResponse>()
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
            Err(Error::PrometheusNotImplemented)
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
            Ok(timeseries)
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
            Ok(timeseries)
        }
        "object" => {
            tracing::error!("Object type not implemented: {}", resp.data.result_type);
            Err(Error::PrometheusNotImplemented)
        }
        _ => {
            tracing::error!("Prometheus invalid result type: {}", resp.data.result_type);
            Err(Error::PrometheusInvalidResultType)
        }
    }
}

// http://192.168.178.96:9090/api/v1/series?match[]={__name__=~"scrape_service_duration_b7091cb8_01f8_4099_b138_5ee6771c5f03_.*"}
pub(crate) async fn request_device_query_operations(
    prometheus_url: &str,
    id: &str,
) -> Result<Vec<String>> {
    let client = Client::new();
    let url = format!("{}/api/v1/series", prometheus_url.trim_end_matches('/'));

    let resp = client
        .get(url)
        .query(&[("match[]", &format!("scrape_service_duration_{id}_.*"))])
        .send()
        .await
        .map_err(Error::Prometheus)?
        .json::<PrometheusMetricNameResponse>()
        .await
        .map_err(Error::PrometheusJson)?;

    Ok(resp
        .data
        .iter()
        .map(|series| series.name.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect())
}
