use greenhouse_core::data_storage_service_dto::diary_dtos::{
    post_diary_entry::PostDiaryEntryDtoRequest,
    put_diary_entry::PutDiaryEntryDtoRequest,
};
use mockito::{mock, Matcher};
use uuid::Uuid;

// Import the service from the main crate
use api_web::diary::service;

#[tokio::test]
async fn test_create_diary_entry_with_alerts() {
    let mock_server = mockito::Server::new();
    let base_url = mock_server.url();

    // Create a mock for the POST request
    let _m = mock("POST", "/data-storage/diary")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("{}")
        .match_body(Matcher::Json(serde_json::json!({
            "date": "2023-01-01T00:00:00Z",
            "title": "Test Diary Entry",
            "content": "Test Content",
            "alert_ids": ["123e4567-e89b-12d3-a456-426614174000"]
        })))
        .create();

    // Create a diary entry with an alert
    let entry = PostDiaryEntryDtoRequest {
        date: "2023-01-01T00:00:00Z".to_string(),
        title: "Test Diary Entry".to_string(),
        content: "Test Content".to_string(),
        alert_ids: Some(vec!["123e4567-e89b-12d3-a456-426614174000".to_string()]),
    };

    // Call the service
    let result = service::create_diary_entry(&base_url, entry).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_diary_entry_with_alerts() {
    let mock_server = mockito::Server::new();
    let base_url = mock_server.url();
    let diary_id = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();

    // Create a mock for the PUT request
    let _m = mock("PUT", "/data-storage/diary/123e4567-e89b-12d3-a456-426614174000")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("{}")
        .match_body(Matcher::Json(serde_json::json!({
            "date": "2023-01-01T00:00:00Z",
            "title": "Updated Diary Entry",
            "content": "Updated Content",
            "alert_ids": ["123e4567-e89b-12d3-a456-426614174001"]
        })))
        .create();

    // Update a diary entry with an alert
    let update = PutDiaryEntryDtoRequest {
        date: "2023-01-01T00:00:00Z".to_string(),
        title: "Updated Diary Entry".to_string(),
        content: "Updated Content".to_string(),
        alert_ids: Some(vec!["123e4567-e89b-12d3-a456-426614174001".to_string()]),
    };

    // Call the service
    let result = service::update_diary_entry(&base_url, diary_id, update).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_diary_entry_with_alerts() {
    let mock_server = mockito::Server::new();
    let base_url = mock_server.url();
    let diary_id = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();

    // Create a mock for the GET request
    let _m = mock("GET", "/data-storage/diary/123e4567-e89b-12d3-a456-426614174000")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "date": "2023-01-01T00:00:00Z",
            "title": "Test Diary Entry",
            "content": "Test Content",
            "created_at": "2023-01-01T00:00:00Z",
            "updated_at": "2023-01-01T00:00:00Z",
            "alert_ids": ["123e4567-e89b-12d3-a456-426614174001"]
        }"#)
        .create();

    // Call the service
    let result = service::get_diary_entry(&base_url, diary_id).await;
    assert!(result.is_ok());

    let entry = result.unwrap();
    assert_eq!(entry.id, "123e4567-e89b-12d3-a456-426614174000");
    assert_eq!(entry.title, "Test Diary Entry");
    assert_eq!(entry.content, "Test Content");
    assert_eq!(entry.alert_ids.len(), 1);
    assert_eq!(entry.alert_ids[0], "123e4567-e89b-12d3-a456-426614174001");
}

#[tokio::test]
async fn test_get_diary() {
    let mock_server = mockito::Server::new();
    let base_url = mock_server.url();
    let start = "2023-01-01T00:00:00Z";
    let end = "2023-01-02T00:00:00Z";

    // Create a mock for the GET request
    let _m = mock("GET", "/data-storage/diary/2023-01-01T00:00:00Z/2023-01-02T00:00:00Z")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "entries": [
                {
                    "id": "123e4567-e89b-12d3-a456-426614174000",
                    "date": "2023-01-01T00:00:00Z",
                    "title": "Test Diary Entry 1",
                    "content": "Test Content 1",
                    "created_at": "2023-01-01T00:00:00Z",
                    "updated_at": "2023-01-01T00:00:00Z",
                    "alert_ids": ["123e4567-e89b-12d3-a456-426614174001"]
                },
                {
                    "id": "123e4567-e89b-12d3-a456-426614174002",
                    "date": "2023-01-01T12:00:00Z",
                    "title": "Test Diary Entry 2",
                    "content": "Test Content 2",
                    "created_at": "2023-01-01T12:00:00Z",
                    "updated_at": "2023-01-01T12:00:00Z",
                    "alert_ids": []
                }
            ]
        }"#)
        .create();

    // Call the service
    let result = service::get_diary(&base_url, start.to_string(), end.to_string()).await;
    assert!(result.is_ok());

    let diary = result.unwrap();
    assert_eq!(diary.entries.len(), 2);
    
    // Check the first entry with an alert
    assert_eq!(diary.entries[0].id, "123e4567-e89b-12d3-a456-426614174000");
    assert_eq!(diary.entries[0].title, "Test Diary Entry 1");
    assert_eq!(diary.entries[0].alert_ids.len(), 1);
    
    // Check the second entry without alerts
    assert_eq!(diary.entries[1].id, "123e4567-e89b-12d3-a456-426614174002");
    assert_eq!(diary.entries[1].title, "Test Diary Entry 2");
    assert_eq!(diary.entries[1].alert_ids.len(), 0);
} 