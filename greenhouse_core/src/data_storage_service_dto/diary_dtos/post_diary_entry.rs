use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PostDiaryEntryDtoRequest {
    pub date: String,
    pub title: String,
    pub content: String,
}
