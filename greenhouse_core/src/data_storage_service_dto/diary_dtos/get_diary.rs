use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

use super::get_diary_entry::DiaryEntryResponseDto;

#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
pub struct GetDiaryResponseDto {
    pub entries: Vec<DiaryEntryResponseDto>,
}
