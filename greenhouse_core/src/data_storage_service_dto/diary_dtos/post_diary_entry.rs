pub struct PostDiaryEntryDtoRequest {
    pub date: String,
    pub title: String,
    pub content: String,
}

pub struct PostDiaryEntryDtoResponse {
    pub id: String,
    pub date: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}
