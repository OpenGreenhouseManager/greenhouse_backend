use serde::{Deserialize, Serialize};

/// Request body for adding a tag to an existing diary entry.
#[derive(Serialize, Deserialize, Debug)]
pub struct PostDiaryTagDtoRequest {
    /// The tag string to attach (e.g. `"harvest"`, `"pest-control"`).
    pub tag_name: String,
}
