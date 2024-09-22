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
    pub name: String,
    pub value: String,
    pub note: Option<String>,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub data_source_id: Uuid,
}

impl Alert {
    pub fn new(
        severity: Severity,
        name: &str,
        value: &str,
        note: Option<&str>,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        data_source_id: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            severity,
            name: String::from(name),
            value: String::from(value),
            note: note.map(String::from),
            start_at,
            end_at,
            data_source_id,
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

    pub async fn find_by_data_source_id(data_source_id: Uuid, pool: &Pool) -> Result<Vec<Self>> {
        let mut conn = pool.get().await.map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
        alert::table
            .filter(alert::data_source_id.eq(data_source_id))
            .load(&mut conn)
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::FindError
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
