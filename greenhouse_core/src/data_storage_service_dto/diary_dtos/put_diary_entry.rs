use serde::{Deserialize, Serialize};

/// Request body for updating an existing diary entry.
///
/// All fields are required; partial updates are not supported.
#[derive(Serialize, Deserialize, Debug)]
pub struct PutDiaryEntryDtoRequest {
    /// Updated calendar date (ISO 8601 date string, e.g. `"2024-06-01"`).
    pub date: String,
    /// Updated title.
    pub title: String,
    /// Updated content body.
    pub content: String,
}
