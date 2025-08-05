use crate::database::schema;
use diesel::prelude::*;
use serde::Deserialize;

#[derive(Debug, Queryable, Selectable, Deserialize, Insertable)]
#[diesel(table_name = schema::scripting_device)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct ScriptingDevice {
    pub(crate) scriptig_key: String,
}
