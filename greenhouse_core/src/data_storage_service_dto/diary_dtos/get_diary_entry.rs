use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

/// A single diary entry returned by the API.
#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
pub struct DiaryEntryResponseDto {
    /// Unique identifier (UUID) for this entry.
    pub id: String,
    /// Calendar date of the observation (ISO 8601 date string, e.g. `"2024-06-01"`).
    pub date: String,
    /// Short title or headline for the entry.
    pub title: String,
    /// Full text body of the entry.
    pub content: String,
    /// ISO 8601 timestamp of when the entry was created.
    pub created_at: String,
    /// ISO 8601 timestamp of the most recent update to this entry.
    pub updated_at: String,
    /// Tags attached to this entry (may be empty).
    #[serde(default)]
    pub tags: Vec<String>,
}
