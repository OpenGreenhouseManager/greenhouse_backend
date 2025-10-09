use super::aggrigated_alert::AggrigatedAlert;
use super::severity_models::Severity;
use super::{Error, Result, schema::alert};
use crate::Pool;
use chrono::{DateTime, Utc};
use diesel::*;
use diesel_async::RunQueryDsl;
use greenhouse_core::data_storage_service_dto::alert_dto::{
    alert::AlertDto,
    post_create_alert::CreateAlertDto,
    query::{AlertQuery, IntervalQuery},
};
use uuid::Uuid;
#[derive(Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::database::schema::alert)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct Alert {
    pub(crate) id: Uuid,
    pub(crate) severity: Severity,
    pub(crate) identifier: String,
    pub(crate) value: String,
    pub(crate) note: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) datasource_id: Uuid,
}

impl Alert {
    pub(crate) async fn create(alert: CreateAlertDto, pool: &Pool) -> Result<Self> {
        let alert = Self {
            id: Uuid::new_v4(),
            severity: alert.severity.into(),
            identifier: alert.identifier.parse().map_err(|e| {
                sentry::capture_error(&e);
                Error::Creation
            })?,
            value: alert.value.unwrap_or_default(),
            note: alert.note,
            created_at: Utc::now(),
            datasource_id: alert.datasource_id.parse().map_err(|e| {
                sentry::capture_error(&e);
                Error::Creation
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
                Error::Creation
            })?;
        Ok(alert)
    }

    pub(crate) async fn query(alert_query: AlertQuery, pool: &Pool) -> Result<Vec<Self>> {
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
            let s: Severity = severity.into();
            query = query.filter(alert::severity.eq(s));
        }
        if let Some(identifier) = alert_query.identifier {
            query = query.filter(alert::identifier.eq(identifier));
        }
        query.load(&mut conn).await.map_err(|e| {
            sentry::capture_error(&e);
            Error::Find
        })
    }

    pub(crate) async fn aggrigate(
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
                Error::Find
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
