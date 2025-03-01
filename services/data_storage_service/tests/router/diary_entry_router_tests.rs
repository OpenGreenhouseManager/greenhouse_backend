use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::Router,
};
use bb8::Pool;
use chrono::Utc;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use greenhouse_core::data_storage_service_dto::alert_dto::{
    alert::Severity as DtoSeverity,
    post_create_alert::CreateAlertDto,
};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

// Import the necessary modules from the main crate
use data_storage_service::{
    database::{
        alert_models::Alert,
        diary_entry_models::DiaryEntry,
    },
    router::diary_entry_router,
    AppState,
};

// Helper function to create a test app with a connection pool
async fn test_app() -> Router {
    // Create a connection pool for testing
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/greenhouse_test".to_string());
    
    let manager = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
    let pool = bb8::Pool::builder()
        .build(manager)
        .await
        .expect("Failed to create connection pool");
    
    let state = AppState {
        config: (),
        pool,
    };
    
    diary_entry_router::routes(state)
}

// Helper function to create a test alert
async fn create_test_alert(pool: &Pool<AsyncDieselConnectionManager<diesel_async::AsyncPgConnection>>) -> Uuid {
    let alert_dto = CreateAlertDto {
        severity: DtoSeverity::Info,
        identifier: "test_alert".to_string(),
        value: Some("test_value".to_string()),
        note: Some("test_note".to_string()),
        datasource_id: Uuid::new_v4().to_string(),
    };
    
    let alert = Alert::create(alert_dto, pool).await.expect("Failed to create test alert");
    alert.id
}

// Helper function to clean up test data
async fn cleanup_test_data(pool: &Pool<AsyncDieselConnectionManager<diesel_async::AsyncPgConnection>>, diary_id: Uuid) {
    let entry = DiaryEntry::find_by_id(diary_id, pool).await.expect("Failed to find diary entry");
    entry.delete(pool).await.expect("Failed to delete diary entry");
}

#[tokio::test]
async fn test_create_diary_entry() {
    // Arrange
    let app = test_app().await;
    
    // Create a diary entry request
    let diary_entry = json!({
        "date": Utc::now().to_rfc3339(),
        "title": "Test Diary Entry",
        "content": "This is a test diary entry"
    });
    
    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/")
                .header("Content-Type", "application/json")
                .body(Body::from(diary_entry.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    
    // Get the response body
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();
    
    // Verify the response contains the expected fields
    assert!(response_json.get("id").is_some());
    assert_eq!(response_json["title"], "Test Diary Entry");
    assert_eq!(response_json["content"], "This is a test diary entry");
    assert!(response_json["alert_ids"].as_array().unwrap().is_empty());
    
    // Clean up
    let diary_id = Uuid::parse_str(response_json["id"].as_str().unwrap()).unwrap();
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/greenhouse_test".to_string());
    let manager = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
    let pool = bb8::Pool::builder()
        .build(manager)
        .await
        .expect("Failed to create connection pool");
    cleanup_test_data(&pool, diary_id).await;
}

#[tokio::test]
async fn test_create_diary_entry_with_alerts() {
    // Arrange
    let app = test_app().await;
    
    // Create a test alert
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/greenhouse_test".to_string());
    let manager = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
    let pool = bb8::Pool::builder()
        .build(manager)
        .await
        .expect("Failed to create connection pool");
    
    let alert_id = create_test_alert(&pool).await;
    
    // Create a diary entry request with the alert
    let diary_entry = json!({
        "date": Utc::now().to_rfc3339(),
        "title": "Test Diary Entry with Alert",
        "content": "This is a test diary entry with an alert",
        "alert_ids": [alert_id.to_string()]
    });
    
    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/")
                .header("Content-Type", "application/json")
                .body(Body::from(diary_entry.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    
    // Get the response body
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();
    
    // Verify the response contains the expected fields
    assert!(response_json.get("id").is_some());
    assert_eq!(response_json["title"], "Test Diary Entry with Alert");
    assert_eq!(response_json["content"], "This is a test diary entry with an alert");
    
    // Verify the alert is linked
    let alert_ids = response_json["alert_ids"].as_array().unwrap();
    assert_eq!(alert_ids.len(), 1);
    assert_eq!(alert_ids[0], alert_id.to_string());
    
    // Clean up
    let diary_id = Uuid::parse_str(response_json["id"].as_str().unwrap()).unwrap();
    cleanup_test_data(&pool, diary_id).await;
}

#[tokio::test]
async fn test_get_diary_entry() {
    // Arrange
    let app = test_app().await;
    
    // Create a test diary entry
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/greenhouse_test".to_string());
    let manager = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
    let pool = bb8::Pool::builder()
        .build(manager)
        .await
        .expect("Failed to create connection pool");
    
    let mut diary_entry = DiaryEntry::new(
        Utc::now(),
        "Test Get Diary Entry",
        "This is a test for getting a diary entry",
    );
    diary_entry.flush(&pool).await.expect("Failed to create diary entry");
    let diary_id = diary_entry.get_id();
    
    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/{}", diary_id))
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    
    // Get the response body
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();
    
    // Verify the response contains the expected fields
    assert_eq!(response_json["id"], diary_id.to_string());
    assert_eq!(response_json["title"], "Test Get Diary Entry");
    assert_eq!(response_json["content"], "This is a test for getting a diary entry");
    
    // Clean up
    cleanup_test_data(&pool, diary_id).await;
}

#[tokio::test]
async fn test_update_diary_entry() {
    // Arrange
    let app = test_app().await;
    
    // Create a test diary entry
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/greenhouse_test".to_string());
    let manager = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
    let pool = bb8::Pool::builder()
        .build(manager)
        .await
        .expect("Failed to create connection pool");
    
    let mut diary_entry = DiaryEntry::new(
        Utc::now(),
        "Original Title",
        "Original Content",
    );
    diary_entry.flush(&pool).await.expect("Failed to create diary entry");
    let diary_id = diary_entry.get_id();
    
    // Create an update request
    let update = json!({
        "date": Utc::now().to_rfc3339(),
        "title": "Updated Title",
        "content": "Updated Content"
    });
    
    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/{}", diary_id))
                .header("Content-Type", "application/json")
                .body(Body::from(update.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    
    // Get the response body
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();
    
    // Verify the response contains the updated fields
    assert_eq!(response_json["id"], diary_id.to_string());
    assert_eq!(response_json["title"], "Updated Title");
    assert_eq!(response_json["content"], "Updated Content");
    
    // Clean up
    cleanup_test_data(&pool, diary_id).await;
}

#[tokio::test]
async fn test_update_diary_entry_with_alerts() {
    // Arrange
    let app = test_app().await;
    
    // Create a test diary entry and alert
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/greenhouse_test".to_string());
    let manager = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
    let pool = bb8::Pool::builder()
        .build(manager)
        .await
        .expect("Failed to create connection pool");
    
    let mut diary_entry = DiaryEntry::new(
        Utc::now(),
        "Original Title",
        "Original Content",
    );
    diary_entry.flush(&pool).await.expect("Failed to create diary entry");
    let diary_id = diary_entry.get_id();
    
    let alert_id = create_test_alert(&pool).await;
    
    // Create an update request with the alert
    let update = json!({
        "date": Utc::now().to_rfc3339(),
        "title": "Updated Title with Alert",
        "content": "Updated Content with Alert",
        "alert_ids": [alert_id.to_string()]
    });
    
    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/{}", diary_id))
                .header("Content-Type", "application/json")
                .body(Body::from(update.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    
    // Get the response body
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();
    
    // Verify the response contains the updated fields
    assert_eq!(response_json["id"], diary_id.to_string());
    assert_eq!(response_json["title"], "Updated Title with Alert");
    assert_eq!(response_json["content"], "Updated Content with Alert");
    
    // Verify the alert is linked
    let alert_ids = response_json["alert_ids"].as_array().unwrap();
    assert_eq!(alert_ids.len(), 1);
    assert_eq!(alert_ids[0], alert_id.to_string());
    
    // Now update again to remove the alert
    let update_no_alerts = json!({
        "date": Utc::now().to_rfc3339(),
        "title": "Updated Title No Alert",
        "content": "Updated Content No Alert",
        "alert_ids": []
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/{}", diary_id))
                .header("Content-Type", "application/json")
                .body(Body::from(update_no_alerts.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Get the response body
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();
    
    // Verify the alert is no longer linked
    let alert_ids = response_json["alert_ids"].as_array().unwrap();
    assert_eq!(alert_ids.len(), 0);
    
    // Clean up
    cleanup_test_data(&pool, diary_id).await;
}

#[tokio::test]
async fn test_get_diary() {
    // Arrange
    let app = test_app().await;
    
    // Create test diary entries
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/greenhouse_test".to_string());
    let manager = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
    let pool = bb8::Pool::builder()
        .build(manager)
        .await
        .expect("Failed to create connection pool");
    
    let now = Utc::now();
    let yesterday = now - chrono::Duration::days(1);
    let tomorrow = now + chrono::Duration::days(1);
    
    // Create entries for yesterday, today, and tomorrow
    let mut entry1 = DiaryEntry::new(
        yesterday,
        "Yesterday's Entry",
        "This is yesterday's entry",
    );
    entry1.flush(&pool).await.expect("Failed to create entry1");
    
    let mut entry2 = DiaryEntry::new(
        now,
        "Today's Entry",
        "This is today's entry",
    );
    entry2.flush(&pool).await.expect("Failed to create entry2");
    
    let mut entry3 = DiaryEntry::new(
        tomorrow,
        "Tomorrow's Entry",
        "This is tomorrow's entry",
    );
    entry3.flush(&pool).await.expect("Failed to create entry3");
    
    // Get entries for today and tomorrow
    let start = now.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string();
    let end = tomorrow.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string();
    
    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/{}/{}", start, end))
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    
    // Get the response body
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();
    
    // Verify we got entries for today and tomorrow (2 entries)
    let entries = response_json["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    
    // Verify the entries have the expected titles
    let titles: Vec<&str> = entries
        .iter()
        .map(|entry| entry["title"].as_str().unwrap())
        .collect();
    
    assert!(titles.contains(&"Today's Entry"));
    assert!(titles.contains(&"Tomorrow's Entry"));
    assert!(!titles.contains(&"Yesterday's Entry"));
    
    // Clean up
    cleanup_test_data(&pool, entry1.get_id()).await;
    cleanup_test_data(&pool, entry2.get_id()).await;
    cleanup_test_data(&pool, entry3.get_id()).await;
} 