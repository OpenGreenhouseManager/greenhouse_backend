pub struct PutDiaryEntryDtoRequest {
    date: String,
    title: String,
    content: String,
}

pub struct PutDiaryEntryDtoResponse {
    id: String,
    date: String,
    title: String,
    content: String,
    created_at: String,
    updated_at: String,
}
