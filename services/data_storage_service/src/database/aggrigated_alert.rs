use super::severity_models::Severity;
use chrono::{DateTime, Utc};
use diesel::deserialize::FromSqlRow;
use greenhouse_core::data_storage_service_dto::alert_dto::get_aggrigated_alert::AggrigatedAlertDto;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, FromSqlRow, Eq, Deserialize)]
pub(crate) struct AggrigatedAlert {
    pub(crate) datasource_id: Uuid,
    pub(crate) severity: Severity,
    pub(crate) count: i64,
    pub(crate) identifier: String,
    pub(crate) first_date: String,
    pub(crate) last_date: String,
}

impl AggrigatedAlert {
    pub(crate) fn new(
        datasource_id: Uuid,
        severity: Severity,
        count: i64,
        identifier: String,
        first_date: Option<DateTime<Utc>>,
        last_date: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            datasource_id,
            severity,
            count,
            identifier,
            first_date: first_date
                .map(|d| d.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string())
                .unwrap_or_default(),
            last_date: last_date
                .map(|d| d.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string())
                .unwrap_or_default(),
        }
    }
}

impl From<AggrigatedAlert> for AggrigatedAlertDto {
    fn from(alert: AggrigatedAlert) -> Self {
        Self {
            count: alert.count,
            identifier: alert.identifier,
            severity: alert.severity.into(),
            source: alert.datasource_id.into(),
            first: alert.first_date,
            last: alert.last_date,
        }
    }
}
