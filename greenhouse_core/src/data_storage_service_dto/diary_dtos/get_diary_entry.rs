use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
pub struct DiaryEntryResponseDto {
    pub id: String,
    pub date: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}
