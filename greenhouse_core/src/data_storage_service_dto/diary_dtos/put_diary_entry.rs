pub struct PutDiaryEntryDtoRequest {
    pub date: String,
    pub title: String,
    pub content: String,
}

pub struct PutDiaryEntryDtoResponse {
    pub id: String,
    pub date: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}
