use super::error::Result;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use greenhouse_core::data_storage_service_dto::diary_dtos::{
    get_diary::GetDiaryResponseDto, get_diary_entry::DiaryEntryResponseDto,
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
        .route("/:id", get(get_diary_entry))
        .route("/:start/:end", get(get_diary))
        .with_state(state)
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
    let start = chrono::NaiveDateTime::parse_from_str(&start, "%Y-%m-%d %H:%M:%S")
        .map_err(|_| Error::TimeError)?;
    let end = chrono::NaiveDateTime::parse_from_str(&end, "%Y-%m-%d %H:%M:%S")
        .map_err(|_| Error::TimeError)?;
    let entries = DiaryEntry::find_by_date_range(start, end, &pool).await?;
    let response: GetDiaryResponseDto = GetDiaryResponseDto {
        entries: entries.into_iter().map(|entry| entry.into()).collect(),
    };
    Ok(Json(response))
}
