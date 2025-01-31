use std::io::Write;

use crate::database::schema::sql_types;
use deserialize::FromSql;
use diesel::deserialize::FromSqlRow;
use diesel::*;
use pg::{Pg, PgValue};
use serde::Deserialize;
use serialize::{IsNull, Output, ToSql};

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
