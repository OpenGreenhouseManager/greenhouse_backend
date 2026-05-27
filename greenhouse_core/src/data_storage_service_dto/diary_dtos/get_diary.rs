use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

use super::get_diary_entry::DiaryEntryResponseDto;

/// All diary entries returned by the list endpoint.
#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
pub struct GetDiaryResponseDto {
    /// Ordered list of diary entries (newest first by convention).
    pub entries: Vec<DiaryEntryResponseDto>,
}
