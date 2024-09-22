use super::{schema::alert, Error, Result};
use crate::{database::schema::sql_types, Pool};
use chrono::{DateTime, Utc};
use deserialize::FromSql;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::*;
use diesel::{deserialize::FromSqlRow, expression::AsExpression};
use diesel_async::RunQueryDsl;
use std::io::Write;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AlertQuery {
    start: Option<String>,
    end: Option<String>,
    datasource_id: Option<String>,
    severity: Option<String>,
    identifier: Option<String>,
}

#[derive(Debug, Clone, PartialEq, FromSqlRow, AsExpression, Eq)]
#[diesel(sql_type = sql_types::Severity)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

impl ToSql<sql_types::Severity, Pg> for Severity {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            Severity::Info => out.write_all(b"info")?,
            Severity::Warning => out.write_all(b"warning")?,
            Severity::Error => out.write_all(b"error")?,
            Severity::Critical => out.write_all(b"critical")?,
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
            b"critical" => Ok(Severity::Critical),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Clone, Queryable, Selectable)]
#[diesel(table_name = crate::database::schema::alert)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Alert {
    pub id: Uuid,
    pub severity: Severity,
    pub identifier: Uuid,
    pub value: String,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub datasource_id: Uuid,
}

impl Alert {
    pub fn new(
        severity: Severity,
        identifier: Uuid,
        value: &str,
        note: Option<&str>,
        created_at: DateTime<Utc>,
        datasource_id: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            severity,
            identifier,
            value: String::from(value),
            note: note.map(String::from),
            created_at,
            datasource_id,
        }
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

    pub async fn query(query: AlertQuery, pool: &Pool) -> Result<Vec<Self>> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        let mut query = alert::table.into_boxed();
        if let Some(start) = query.start {
            query = query.filter(alert::created_at.ge(start));
        }
        if let Some(end) = query.end {
            query = query.filter(alert::created_at.le(end));
        }
        if let Some(datasource_id) = query.datasource_id {
            query = query.filter(alert::datasource_id.eq(datasource_id));
        }
        if let Some(severity) = query.severity {
            query = query.filter(alert::severity.eq(severity));
        }
        if let Some(identifier) = query.identifier {
            query = query.filter(alert::identifier.eq(identifier));
        }
        query.load(&mut conn).await.map_err(|e| {
            sentry::capture_error(&e);
            Error::FindError
        })
    }

    pub async fn aggrigate(pool: &Pool) -> Result<Vec<AlertAggrigatedDto>> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        let a = alert::table
            .select((
                diesel::dsl::count_star(),
                alert::identifier,
                alert::datasource_id,
                alert::severity,
                diesel::dsl::max(alert::created_at),
                diesel::dsl::min(alert::created_at),
            ))
            .group_by((alert::identifier, alert::datasource_id, alert::severity))
            .load(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::FindError
            })?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
