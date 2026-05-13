use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PostDiaryTagDtoRequest {
    pub tag_name: String,
}
