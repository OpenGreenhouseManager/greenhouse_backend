use super::{schema::alert, Error, Result};
use crate::{database::schema::sql_types, Pool};
use chrono::{DateTime, Utc};
use deserialize::FromSql;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::*;
use diesel::{deserialize::FromSqlRow, expression::AsExpression};
use diesel_async::RunQueryDsl;
use greenhouse_core::data_storage_service_dto::alert_dto::alert::AlertDto;
use greenhouse_core::data_storage_service_dto::alert_dto::get_aggrigated_alert::AlertAggrigatedDto;
use greenhouse_core::data_storage_service_dto::alert_dto::post_create_alert::CreateAlertDto;
use serde::Deserialize;
use std::io::Write;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AlertQuery {
    severity: Option<Severity>,
    identifier: Option<String>,
    created_at: Option<DateTime<Utc>>,
    datasource_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct IntervalQuery {
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, FromSqlRow, Eq, Deserialize)]
pub struct AggrigatedAlert {
    pub datasource_id: Uuid,
    pub severity: Severity,
    pub count: i64,
    pub identifier: String,
    pub first_date: String,
    pub last_date: String,
}

impl AggrigatedAlert {
    pub fn new(
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

#[derive(Debug, Clone, PartialEq, FromSqlRow, AsExpression, Eq, Deserialize)]
#[diesel(sql_type = sql_types::Severity)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Fatal,
}

impl ToSql<sql_types::Severity, Pg> for Severity {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            Severity::Info => out.write_all(b"info")?,
            Severity::Warning => out.write_all(b"warning")?,
            Severity::Error => out.write_all(b"error")?,
            Severity::Fatal => out.write_all(b"fatal")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<sql_types::Severity, Pg> for Severity {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"info" => Ok(Severity::Info),
            b"warning" => Ok(Severity::Warning),
            b"error" => Ok(Severity::Error),
            b"fatal" => Ok(Severity::Fatal),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::database::schema::alert)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Alert {
    pub id: Uuid,
    pub severity: Severity,
    pub identifier: String,
    pub value: String,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub datasource_id: Uuid,
}

impl Alert {
    pub async fn create(alert: CreateAlertDto, pool: &Pool) -> Result<Self> {
        let now = Utc::now();
        let alert = Self {
            id: Uuid::new_v4(),
            severity: alert.severity.into(),
            identifier: alert.identifier.parse().map_err(|e| {
                sentry::capture_error(&e);
                Error::CreationError
            })?,
            value: alert.value.unwrap_or_default(),
            note: alert.note,
            created_at: now,
            datasource_id: alert.datasource_id.parse().map_err(|e| {
                sentry::capture_error(&e);
                Error::CreationError
            })?,
        };
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        diesel::insert_into(alert::table)
            .values(&alert)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::CreationError
            })?;
        Ok(alert)
    }

    pub async fn find_by_id(id: Uuid, pool: &Pool) -> Result<Self> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        alert::table
            .filter(alert::id.eq(id))
            .first(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::FindError
            })
    }

    pub async fn find_by_data_source_id(datasource_id: Uuid, pool: &Pool) -> Result<Vec<Self>> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        alert::table
            .filter(alert::datasource_id.eq(datasource_id))
            .load(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::FindError
            })
    }

    pub async fn query(alert_query: AlertQuery, pool: &Pool) -> Result<Vec<Self>> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        let mut query = alert::table.into_boxed();
        if let Some(start) = alert_query.created_at {
            query = query.filter(alert::created_at.ge(start));
        }
        if let Some(datasource_id) = alert_query.datasource_id {
            query = query.filter(alert::datasource_id.eq(datasource_id));
        }
        if let Some(severity) = alert_query.severity {
            query = query.filter(alert::severity.eq(severity));
        }
        if let Some(identifier) = alert_query.identifier {
            query = query.filter(alert::identifier.eq(identifier));
        }
        query.load(&mut conn).await.map_err(|e| {
            sentry::capture_error(&e);
            Error::FindError
        })
    }

    pub async fn aggrigate(
        interval_query: IntervalQuery,
        pool: &Pool,
    ) -> Result<Vec<AggrigatedAlert>> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;

        let mut query = alert::table
            .group_by((alert::datasource_id, alert::severity, alert::identifier))
            .select((
                alert::datasource_id,
                alert::severity,
                alert::identifier,
                diesel::dsl::count(alert::id),
                diesel::dsl::min(alert::created_at),
                diesel::dsl::max(alert::created_at),
            ))
            .into_boxed();

        if let Some(start) = interval_query.start {
            query = query.filter(alert::created_at.ge(start));
        }
        if let Some(end) = interval_query.end {
            query = query.filter(alert::created_at.le(end));
        }

        let query = query
            .load::<(
                Uuid,
                Severity,
                String,
                i64,
                Option<DateTime<Utc>>,
                Option<DateTime<Utc>>,
            )>(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::FindError
            })?;

        Ok(query
            .into_iter()
            .map(
                |(datasource_id, severity, identifier, count, first_date, last_date)| {
                    AggrigatedAlert::new(
                        datasource_id,
                        severity,
                        count,
                        identifier,
                        first_date,
                        last_date,
                    )
                },
            )
            .collect())
    }
}

impl From<AggrigatedAlert> for AlertAggrigatedDto {
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

impl From<Alert> for AlertDto {
    fn from(alert: Alert) -> Self {
        Self {
            id: alert.id.to_string(),
            severity: alert.severity.into(),
            identifier: alert.identifier.to_string(),
            value: alert.value,
            note: alert.note,
            created_at: alert.created_at.to_rfc3339(),
            datasource_id: alert.datasource_id.to_string(),
        }
    }
}

impl From<Severity> for greenhouse_core::data_storage_service_dto::alert_dto::alert::Severity {
    fn from(severity: Severity) -> Self {
        match severity {
            Severity::Info => Self::Info,
            Severity::Warning => Self::Warning,
            Severity::Error => Self::Error,
            Severity::Fatal => Self::Fatal,
        }
    }
}

impl From<greenhouse_core::data_storage_service_dto::alert_dto::alert::Severity> for Severity {
    fn from(
        severity: greenhouse_core::data_storage_service_dto::alert_dto::alert::Severity,
    ) -> Self {
        match severity {
            greenhouse_core::data_storage_service_dto::alert_dto::alert::Severity::Info => {
                Self::Info
            }
            greenhouse_core::data_storage_service_dto::alert_dto::alert::Severity::Warning => {
                Self::Warning
            }
            greenhouse_core::data_storage_service_dto::alert_dto::alert::Severity::Error => {
                Self::Error
            }
            greenhouse_core::data_storage_service_dto::alert_dto::alert::Severity::Fatal => {
                Self::Fatal
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
