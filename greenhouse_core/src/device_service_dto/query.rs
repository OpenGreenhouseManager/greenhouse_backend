use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct PromQuery {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub sub_property: Option<String>,
    pub step: Option<String>,
}
