use super::error::Result;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use greenhouse_core::data_storage_service_dto::diary_dtos::{
    get_diary::GetDiaryResponseDto, get_diary_entry::DiaryEntryResponseDto,
    post_diary_entry::PostDiaryEntryDtoRequest, put_diary_entry::PutDiaryEntryDtoRequest,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{database::models::DiaryEntry, router::error::Error, AppState};

#[derive(Deserialize)]
pub struct Params {
    start: String,
    end: String,
}

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_diary_entry))
        .route("/:id", put(update_diary_entry))
        .route("/:id", get(get_diary_entry))
        .route("/:start/:end", get(get_diary))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn update_diary_entry(
    State(AppState { config: _, pool }): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update): Json<PutDiaryEntryDtoRequest>,
) -> Result<impl IntoResponse> {
    let mut entry = DiaryEntry::find_by_id(id, &pool).await?;
    entry.title = update.title.clone();
    entry.entry_date = chrono::NaiveDateTime::parse_from_str(&update.date, "%Y-%m-%dT%H:%M:%S%.fZ")
        .map_err(|_| Error::TimeError)?;

    entry.content = update.content.clone();
    entry.flush(&pool).await?;
    let response: DiaryEntryResponseDto = entry.into();
    Ok(Json(response))
}

#[axum::debug_handler]
pub(crate) async fn create_diary_entry(
    State(AppState { config: _, pool }): State<AppState>,
    Json(entry): Json<PostDiaryEntryDtoRequest>,
) -> Result<impl IntoResponse> {
    let mut entry = DiaryEntry::new(
        chrono::NaiveDateTime::parse_from_str(&entry.date, "%Y-%m-%dT%H:%M:%S%.fZ")
            .map_err(|_| Error::TimeError)?,
        &entry.title,
        &entry.content,
    );
    entry.flush(&pool).await?;
    let response: DiaryEntryResponseDto = entry.into();
    Ok(Json(response))
}

#[axum::debug_handler]
pub(crate) async fn get_diary_entry(
    State(AppState { config: _, pool }): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let entry = DiaryEntry::find_by_id(id, &pool).await?;
    let response: DiaryEntryResponseDto = entry.into();
    Ok(Json(response))
}

#[axum::debug_handler]
pub(crate) async fn get_diary(
    State(AppState { config: _, pool }): State<AppState>,
    Path(Params { start, end }): Path<Params>,
) -> Result<impl IntoResponse> {
    let start = chrono::NaiveDateTime::parse_from_str(&start, "%Y-%m-%dT%H:%M:%S%.fZ")
        .map_err(|_| Error::TimeError)?;
    let end = chrono::NaiveDateTime::parse_from_str(&end, "%Y-%m-%dT%H:%M:%S%.fZ")
        .map_err(|_| Error::TimeError)?;
    let entries = DiaryEntry::find_by_date_range(start, end, &pool).await?;
    let response: GetDiaryResponseDto = GetDiaryResponseDto {
        entries: entries.into_iter().map(|entry| entry.into()).collect(),
    };
    Ok(Json(response))
}
