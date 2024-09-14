pub struct PostDiaryEntryDtoRequest {
    date: String,
    title: String,
    content: String,
}

pub struct PostDiaryEntryDtoResponse {
    id: String,
    date: String,
    title: String,
    content: String,
    created_at: String,
    updated_at: String,
}
