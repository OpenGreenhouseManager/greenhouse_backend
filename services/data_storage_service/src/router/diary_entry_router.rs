use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post, put},
};
use chrono::{DateTime, Utc};
use greenhouse_core::data_storage_service_dto::diary_dtos::{
    get_diary::GetDiaryResponseDto, get_diary_entry::DiaryEntryResponseDto,
    post_diary_entry::PostDiaryEntryDtoRequest, put_diary_entry::PutDiaryEntryDtoRequest,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppState,
    database::diary_models::DiaryEntry,
    router::error::{Error, HttpResult},
};

#[derive(Deserialize)]
pub(crate) struct Params {
    start: String,
    end: String,
}

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_diary_entry))
        .route("/{id}", put(update_diary_entry))
        .route("/{id}", get(get_diary_entry))
        .route("/{start}/{end}", get(get_diary))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn update_diary_entry(
    State(AppState { config: _, pool }): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update): Json<PutDiaryEntryDtoRequest>,
) -> HttpResult<impl IntoResponse> {
    let mut entry = DiaryEntry::find_by_id(id, &pool).await?;
    entry.title = update.title.clone();
    entry.entry_date = update.date.parse::<DateTime<Utc>>().map_err(|e| {
        sentry::configure_scope(|scope| {
            let mut map = std::collections::BTreeMap::new();
            map.insert(String::from("time"), update.date.clone().into());

            scope.set_context("time_string", sentry::protocol::Context::Other(map));
        });

        sentry::capture_error(&e);
        Error::TimeError
    })?;

    entry.content = update.content.clone();
    entry.flush(&pool).await?;
    let response: DiaryEntryResponseDto = entry.into();
    Ok(Json(response))
}

#[axum::debug_handler]
pub(crate) async fn create_diary_entry(
    State(AppState { config: _, pool }): State<AppState>,
    Json(entry): Json<PostDiaryEntryDtoRequest>,
) -> HttpResult<impl IntoResponse> {
    let mut entry = DiaryEntry::new(
        entry.date.parse::<DateTime<Utc>>().map_err(|e| {
            sentry::configure_scope(|scope| {
                let mut map = std::collections::BTreeMap::new();
                map.insert(String::from("time"), entry.date.clone().into());

                scope.set_context("time_string", sentry::protocol::Context::Other(map));
            });

            sentry::capture_error(&e);
            Error::TimeError
        })?,
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
) -> HttpResult<impl IntoResponse> {
    let entry = DiaryEntry::find_by_id(id, &pool).await?;
    let response: DiaryEntryResponseDto = entry.into();
    Ok(Json(response))
}

#[axum::debug_handler]
pub(crate) async fn get_diary(
    State(AppState { config: _, pool }): State<AppState>,
    Path(Params { start, end }): Path<Params>,
) -> HttpResult<impl IntoResponse> {
    let start = start.parse::<DateTime<Utc>>().map_err(|e| {
        sentry::configure_scope(|scope| {
            let mut map = std::collections::BTreeMap::new();
            map.insert(String::from("time"), start.clone().into());

            scope.set_context("time_string", sentry::protocol::Context::Other(map));
        });

        sentry::capture_error(&e);
        Error::TimeError
    })?;
    let end = end.parse::<DateTime<Utc>>().map_err(|e| {
        sentry::configure_scope(|scope| {
            let mut map = std::collections::BTreeMap::new();
            map.insert(String::from("time"), end.clone().into());

            scope.set_context("time_string", sentry::protocol::Context::Other(map));
        });

        sentry::capture_error(&e);
        Error::TimeError
    })?;
    let entries = DiaryEntry::find_by_date_range(start, end, &pool).await?;
    let response: GetDiaryResponseDto = GetDiaryResponseDto {
        entries: entries.into_iter().map(|entry| entry.into()).collect(),
    };
    Ok(Json(response))
}
