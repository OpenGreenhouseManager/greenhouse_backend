use std::time::Duration;

use greenhouse_core::{
    data_storage_service_dto::diary_dtos::{
        endpoints, get_diary::GetDiaryResponseDto, get_diary_entry::DiaryEntryResponseDto,
        post_diary_entry::PostDiaryEntryDtoRequest, post_diary_tag::PostDiaryTagDtoRequest,
        put_diary_entry::PutDiaryEntryDtoRequest,
    },
    http_error::ErrorResponseBody,
};
use uuid::Uuid;

use crate::{
    diary::{Error, Result},
    helper::error::ApiError,
};

pub(crate) async fn create_diary_entry(
    base_ulr: &str,
    entry: PostDiaryEntryDtoRequest,
) -> Result<()> {
    let resp = reqwest::Client::new()
        .post(base_ulr.to_string() + endpoints::DIARY)
        .json(&entry)
        .send()
        .await
        .map_err(|e| {

            tracing::error!(
                "Error in post to service: {:?} with entry: {:?} for url {}",
                e,
                entry,
                base_ulr
            );

            Error::Request(e)
        })?;
    if resp.status().is_success() {
        return Ok(());
    }
    Err(Error::Api(ApiError {
        status: resp.status(),
        message: resp
            .json::<ErrorResponseBody>()
            .await
            .map_err(|e| {
                tracing::error!("Error in get to service: {:?}", e);
                Error::Json(e)
            })?
            .error,
    }))
}

pub(crate) async fn update_diary_entry(
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

            tracing::error!(
                "Error in put to service: {:?} with entry: {:?} for url {}",
                e,
                update,
                base_ulr
            );

            Error::Request(e)
        })?;
    if resp.status().is_success() {
        return Ok(());
    }
    Err(Error::Api(ApiError {
        status: resp.status(),
        message: resp
            .json::<ErrorResponseBody>()
            .await
            .map_err(|e| {
                tracing::error!("Error in get to service: {:?}", e);
                Error::Json(e)
            })?
            .error,
    }))
}

pub(crate) async fn get_diary_entry(base_ulr: &str, id: Uuid) -> Result<DiaryEntryResponseDto> {
    let resp = reqwest::Client::new()
        .get(base_ulr.to_string() + endpoints::DIARY + "/" + &id.to_string())
        .send()
        .await
        .map_err(|e| {

            tracing::error!(
                "Error in get to service: {:?} with id: {:?} for url {}",
                e,
                id,
                base_ulr
            );

            Error::Request(e)
        })?;
    if resp.status().is_success() {
        return resp.json().await.map_err(|e| {
            tracing::error!("Error in get to service: {:?}", e,);
            Error::Json(e)
        });
    }
    Err(Error::Api(ApiError {
        status: resp.status(),
        message: resp
            .json::<ErrorResponseBody>()
            .await
            .map_err(|e| {
                tracing::error!("Error in get to service: {:?}", e);
                Error::Json(e)
            })?
            .error,
    }))
}

pub(crate) async fn get_diary(
    base_ulr: &str,
    start: String,
    end: String,
) -> Result<GetDiaryResponseDto> {
    let resp = reqwest::Client::new()
        .get(base_ulr.to_string() + endpoints::DIARY + "/" + &start + "/" + &end)
        .send()
        .await
        .map_err(|e| {

            tracing::error!(
                "Error in get to service: {:?} with start: {:?} and end: {:?} for url {}",
                e,
                start,
                end,
                base_ulr
            );

            Error::Request(e)
        })?;
    if resp.status().is_success() {
        return resp.json().await.map_err(|e| {
            tracing::error!("Error in get to service: {:?}", e,);
            Error::Json(e)
        });
    }
    Err(Error::Api(ApiError {
        status: resp.status(),
        message: resp
            .json::<ErrorResponseBody>()
            .await
            .map_err(|e| {
                tracing::error!("Error in get to service: {:?}", e);
                Error::Json(e)
            })?
            .error,
    }))
}

const DOWNSTREAM_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) async fn add_tag(
    base_ulr: &str,
    entry_id: Uuid,
    body: PostDiaryTagDtoRequest,
) -> Result<()> {
    let resp = reqwest::Client::new()
        .post(base_ulr.to_string() + endpoints::DIARY + "/" + &entry_id.to_string() + "/tags")
        .timeout(DOWNSTREAM_TIMEOUT)
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Error in post to service: {:?} for url {}", e, base_ulr);
            Error::Request(e)
        })?;
    if resp.status().is_success() {
        return Ok(());
    }
    Err(Error::Api(ApiError {
        status: resp.status(),
        message: resp
            .json::<ErrorResponseBody>()
            .await
            .map_err(|e| {
                tracing::error!("Error in get to service: {:?}", e);
                Error::Json(e)
            })?
            .error,
    }))
}

pub(crate) async fn remove_tag(base_ulr: &str, entry_id: Uuid, tag_name: String) -> Result<()> {
    let encoded = urlencoding::encode(&tag_name);
    let resp = reqwest::Client::new()
        .delete(
            base_ulr.to_string()
                + endpoints::DIARY
                + "/"
                + &entry_id.to_string()
                + "/tags/"
                + encoded.as_ref(),
        )
        .timeout(DOWNSTREAM_TIMEOUT)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Error in delete to service: {:?} for url {}", e, base_ulr);
            Error::Request(e)
        })?;
    if resp.status().is_success() {
        return Ok(());
    }
    Err(Error::Api(ApiError {
        status: resp.status(),
        message: resp
            .json::<ErrorResponseBody>()
            .await
            .map_err(|e| {
                tracing::error!("Error in get to service: {:?}", e);
                Error::Json(e)
            })?
            .error,
    }))
}

pub(crate) async fn search_by_tag(base_ulr: &str, tag_name: String) -> Result<GetDiaryResponseDto> {
    let encoded = urlencoding::encode(&tag_name);
    let resp = reqwest::Client::new()
        .get(base_ulr.to_string() + endpoints::DIARY + "/tags/" + encoded.as_ref())
        .timeout(DOWNSTREAM_TIMEOUT)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Error in get to service: {:?} for url {}", e, base_ulr);
            Error::Request(e)
        })?;
    if resp.status().is_success() {
        return resp.json().await.map_err(|e| {
            tracing::error!("Error in get to service: {:?}", e);
            Error::Json(e)
        });
    }
    Err(Error::Api(ApiError {
        status: resp.status(),
        message: resp
            .json::<ErrorResponseBody>()
            .await
            .map_err(|e| {
                tracing::error!("Error in get to service: {:?}", e);
                Error::Json(e)
            })?
            .error,
    }))
}
