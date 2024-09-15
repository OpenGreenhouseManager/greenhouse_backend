use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PutDiaryEntryDtoRequest {
    pub date: String,
    pub title: String,
    pub content: String,
}
