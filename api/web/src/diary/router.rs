use crate::AppState;
use crate::diary::service;
use crate::helper::error::HttpResult;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use greenhouse_core::data_storage_service_dto::diary_dtos::get_diary::GetDiaryResponseDto;
use greenhouse_core::data_storage_service_dto::diary_dtos::get_diary_entry::DiaryEntryResponseDto;
use greenhouse_core::data_storage_service_dto::diary_dtos::post_diary_entry::PostDiaryEntryDtoRequest;
use greenhouse_core::data_storage_service_dto::diary_dtos::post_diary_tag::PostDiaryTagDtoRequest;
use greenhouse_core::data_storage_service_dto::diary_dtos::put_diary_entry::PutDiaryEntryDtoRequest;
use uuid::Uuid;

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_diary_entry))
        .route("/{id}", put(update_diary_entry))
        .route("/{id}", get(get_diary_entry))
        .route("/{start}/{end}", get(get_diary))
        .route("/{id}/tags", post(add_tag_to_entry))
        .route("/{id}/tags/{tag_name}", delete(remove_tag_from_entry))
        .route("/tags/{tag_name}", get(search_entries_by_tag))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn create_diary_entry(
    State(AppState { config }): State<AppState>,
    Json(entry): Json<PostDiaryEntryDtoRequest>,
) -> HttpResult<()> {
    service::create_diary_entry(&config.service_addresses.data_storage_service, entry).await?;
    Ok(())
}

#[axum::debug_handler]
pub(crate) async fn update_diary_entry(
    State(AppState { config }): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update): Json<PutDiaryEntryDtoRequest>,
) -> HttpResult<()> {
    service::update_diary_entry(&config.service_addresses.data_storage_service, id, update).await?;
    Ok(())
}

#[axum::debug_handler]
pub(crate) async fn get_diary_entry(
    State(AppState { config }): State<AppState>,
    Path(id): Path<Uuid>,
) -> HttpResult<DiaryEntryResponseDto> {
    Ok(service::get_diary_entry(&config.service_addresses.data_storage_service, id).await?)
}

#[axum::debug_handler]
pub(crate) async fn get_diary(
    State(AppState { config }): State<AppState>,
    Path((start, end)): Path<(String, String)>,
) -> HttpResult<GetDiaryResponseDto> {
    Ok(service::get_diary(&config.service_addresses.data_storage_service, start, end).await?)
}

#[axum::debug_handler]
pub(crate) async fn add_tag_to_entry(
    State(AppState { config }): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<PostDiaryTagDtoRequest>,
) -> HttpResult<StatusCode> {
    service::add_tag(
        &config.service_addresses.data_storage_service,
        id,
        body.tag_name,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[axum::debug_handler]
pub(crate) async fn remove_tag_from_entry(
    State(AppState { config }): State<AppState>,
    Path((id, tag_name)): Path<(Uuid, String)>,
) -> HttpResult<StatusCode> {
    service::remove_tag(
        &config.service_addresses.data_storage_service,
        id,
        tag_name,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[axum::debug_handler]
pub(crate) async fn search_entries_by_tag(
    State(AppState { config }): State<AppState>,
    Path(tag_name): Path<String>,
) -> HttpResult<GetDiaryResponseDto> {
    Ok(
        service::search_by_tag(&config.service_addresses.data_storage_service, tag_name)
            .await?,
    )
}
