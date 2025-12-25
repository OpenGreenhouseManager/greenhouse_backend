use super::error::{Error, Result};
use greenhouse_core::device_service_dto::{
    get_timeseries::{GetTimeseriesDto, Measurement, TimeseriesDto, Type},
    operations::OperationsDto,
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
) -> Result<GetTimeseriesDto> {
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
        "number" => {
            let timeseries = resp.data.result[0]
                .values
                .iter()
                .map(|result| TimeseriesDto {
                    timestamp: result.0,
                    value: Type::Number(result.1.parse::<f64>().unwrap()),
                })
                .collect::<Vec<TimeseriesDto>>()
                .into();
            Ok(timeseries)
        }
        "boolean" => {
            let timeseries = resp.data.result[0]
                .values
                .iter()
                .map(|result| TimeseriesDto {
                    timestamp: result.0,
                    value: Type::Boolean(result.1.parse::<bool>().unwrap()),
                })
                .collect::<Vec<TimeseriesDto>>()
                .into();
            Ok(timeseries)
        }
        "object" => {
            tracing::error!("Object type not implemented: {}", resp.data.result_type);
            Err(Error::PrometheusNotImplemented)
        }
        "measurement" => {
            let unknown = "unknown".to_string();
            let unit = resp.data.result[0].metric.get("unit").unwrap_or(&unknown);

            let timeseries = resp.data.result[0]
                .values
                .iter()
                .map(|result| TimeseriesDto {
                    timestamp: result.0,
                    value: Type::Measurement(Measurement {
                        value: result.1.parse::<f64>().unwrap(),
                        unit: unit.to_string(),
                    }),
                })
                .collect::<Vec<TimeseriesDto>>()
                .into();
            Ok(timeseries)
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
