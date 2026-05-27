use serde::{Deserialize, Serialize};

/// Request body for creating a new diary entry.
#[derive(Serialize, Deserialize, Debug)]
pub struct PostDiaryEntryDtoRequest {
    /// Calendar date of the observation (ISO 8601 date string, e.g. `"2024-06-01"`).
    pub date: String,
    /// Short title or headline for the entry.
    pub title: String,
    /// Full text body of the entry.
    pub content: String,
}
