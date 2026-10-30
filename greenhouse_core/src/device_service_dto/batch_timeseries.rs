use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{get_timeseries::TimeseriesDto, query::PromQuery};

/// A [`PromQuery`] bound to a specific device.
#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceTimeseriesQuery {
    pub device_id: Uuid,
    #[serde(flatten)]
    pub query: PromQuery,
}

/// Request several timeseries, possibly from different devices.
#[derive(Serialize, Deserialize, Debug)]
pub struct BatchTimeseriesRequestDto {
    pub queries: Vec<DeviceTimeseriesQuery>,
}

/// Results are returned in the same order as the requested queries.
#[derive(Serialize, Deserialize, IntoJsonResponse)]
pub struct BatchTimeseriesResponseDto {
    pub results: Vec<BatchTimeseriesResultDto>,
}

/// Exactly one of `timeseries` or `error` is set, so a single failing
/// query does not fail the whole batch.
#[derive(Serialize, Deserialize)]
pub struct BatchTimeseriesResultDto {
    pub query: DeviceTimeseriesQuery,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeseries: Option<Vec<TimeseriesDto>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl From<Vec<BatchTimeseriesResultDto>> for BatchTimeseriesResponseDto {
    fn from(results: Vec<BatchTimeseriesResultDto>) -> Self {
        Self { results }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device_service_dto::query::Step;

    #[test]
    fn flattened_query_deserializes_with_default_step() {
        let request: BatchTimeseriesRequestDto = serde_json::from_str(
            r#"{"queries":[
                {"device_id":"6f1c2c6e-0d6b-4a7e-9a4a-1c2d3e4f5a6b","sub_property":"temp",
                 "start":"2026-01-01T00:00:00Z","end":"2026-01-02T00:00:00Z","step":"5m"},
                {"device_id":"6f1c2c6e-0d6b-4a7e-9a4a-1c2d3e4f5a6b",
                 "start":"2026-01-01T00:00:00Z","end":"2026-01-02T00:00:00Z"}
            ]}"#,
        )
        .unwrap();

        assert_eq!(
            request.queries[0].query.sub_property.as_deref(),
            Some("temp")
        );
        assert_eq!(request.queries[0].query.step.to_string(), "5m");
        assert_eq!(request.queries[1].query.sub_property, None);
        assert_eq!(request.queries[1].query.step, Step::default());
    }
}
