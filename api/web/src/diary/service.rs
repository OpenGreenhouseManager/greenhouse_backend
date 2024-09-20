use greenhouse_core::data_storage_service_dto::diary_dtos::{
    endpoints, get_diary::GetDiaryResponseDto, get_diary_entry::DiaryEntryResponseDto,
    post_diary_entry::PostDiaryEntryDtoRequest, put_diary_entry::PutDiaryEntryDtoRequest,
};
use uuid::Uuid;

use crate::diary::{Error, Result};

pub async fn create_diary_entry(base_ulr: &str, entry: PostDiaryEntryDtoRequest) -> Result<()> {
    let resp = reqwest::Client::new()
        .post(base_ulr.to_string() + endpoints::DIARY)
        .json(&entry)
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);
            Error::InternalError
        })?;
    resp.error_for_status().map_err(|e| {
        sentry::capture_error(&e);
        Error::InternalError
    })?;

    Ok(())
}

pub async fn update_diary_entry(
    base_ulr: &str,
    id: Uuid,
    update: PutDiaryEntryDtoRequest,
) -> Result<()> {
    let resp = reqwest::Client::new()
        .put(base_ulr.to_string() + endpoints::DIARY + "/" + &id.to_string())
        .json(&update)
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);
            Error::InternalError
        })?;
    resp.error_for_status().map_err(|e| {
        sentry::capture_error(&e);
        Error::InternalError
    })?;

    Ok(())
}

pub async fn get_diary_entry(base_ulr: &str, id: Uuid) -> Result<DiaryEntryResponseDto> {
    let resp = reqwest::Client::new()
        .get(base_ulr.to_string() + endpoints::DIARY + "/" + &id.to_string())
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);
            Error::InternalError
        })?;
    resp.json().await.map_err(|e| {
        sentry::capture_error(&e);
        Error::InternalError
    })
}

pub async fn get_diary(base_ulr: &str, start: String, end: String) -> Result<GetDiaryResponseDto> {
    let resp = reqwest::Client::new()
        .get(base_ulr.to_string() + endpoints::DIARY + "/" + &start + "/" + &end)
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);
            Error::InternalError
        })?;
    resp.json().await.map_err(|e| {
        sentry::capture_error(&e);
        Error::InternalError
    })
}
