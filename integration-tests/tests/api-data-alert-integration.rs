use greenhouse_core::data_storage_service_dto::alert_dto::alert::Severity;
use greenhouse_core::data_storage_service_dto::alert_dto::post_create_alert::CreateAlertDto;
use greenhouse_core::data_storage_service_dto::alert_dto::query::IntervalQuery;
use test_helper::TestContext;
mod test_helper;

#[tokio::test]
async fn test_create_and_filter_alert() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    // Create alert
    let client = reqwest::Client::new();
    let create_alert = CreateAlertDto {
        severity: Severity::Error,
        identifier: String::from("temperature"),
        value: Some(String::from("42.0")),
        note: Some(String::from("High temperature detected")),
        datasource_id: String::from(uuid::Uuid::new_v4()),
    };
    let response = client
        .post("http://localhost:3000/api/alert")
        .json(&create_alert)
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "Failed to create alert: {}",
        response.status()
    );

    // Filter alerts
    let response = client
        .get("http://localhost:3000/api/alert/filter?alert_type=temperature")
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success(), "Failed to filter alerts");
    let alerts: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(!alerts.is_empty(), "No alerts returned");

    context.stop().await;
}

#[tokio::test]
async fn test_alert_subset() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    // Create alert
    let client = reqwest::Client::new();
    let create_alert = CreateAlertDto {
        severity: Severity::Warning,
        identifier: String::from("humidity"),
        value: Some(String::from("15.5")),
        note: Some(String::from("Low humidity detected")),
        datasource_id: String::from(uuid::Uuid::new_v4()),
    };
    let response = client
        .post("http://localhost:3000/api/alert")
        .json(&create_alert)
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success(), "Failed to create alert");

    let q = IntervalQuery {
        start: Some(chrono::Utc::now() - chrono::Duration::days(1)),
        end: Some(chrono::Utc::now()),
    };

    // Get alert subset by interval
    let response = client
        .get("http://localhost:3000/api/alert")
        .query(&q)
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success(), "Failed to get alert subset");
    let alerts: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(!alerts.is_empty(), "No alerts returned in subset");

    context.stop().await;
}

#[tokio::test]
async fn test_create_alert_missing_fields() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    // Missing required field: identifier
    let client = reqwest::Client::new();
    let invalid_alert = serde_json::json!({
        "severity": "Error",
        // "identifier" is missing
        "value": "100.0",
        "note": "Missing identifier",
        "datasource_id": uuid::Uuid::new_v4().to_string()
    });
    let response = client
        .post("http://localhost:3000/api/alert")
        .json(&invalid_alert)
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(
        !response.status().is_success(),
        "Should fail to create alert with missing fields"
    );

    context.stop().await;
}

#[tokio::test]
async fn test_create_alert_invalid_severity() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    // Invalid severity value
    let client = reqwest::Client::new();
    let invalid_alert = serde_json::json!({
        "severity": "NotASeverity",
        "identifier": "temperature",
        "value": "100.0",
        "note": "Invalid severity",
        "datasource_id": uuid::Uuid::new_v4().to_string()
    });
    let response = client
        .post("http://localhost:3000/api/alert")
        .json(&invalid_alert)
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(
        !response.status().is_success(),
        "Should fail to create alert with invalid severity"
    );

    context.stop().await;
}

#[tokio::test]
async fn test_create_alert_unauthorized() {
    let mut context = TestContext::new();
    context.start_all_services().await;

    // No auth token
    let client = reqwest::Client::new();
    let create_alert = serde_json::json!({
        "severity": "Error",
        "identifier": "unauthorized",
        "value": "1.0",
        "note": "No auth",
        "datasource_id": uuid::Uuid::new_v4().to_string()
    });
    let response = client
        .post("http://localhost:3000/api/alert")
        .json(&create_alert)
        .header("Access-Control-Allow-Credentials", "true")
        .send()
        .await
        .unwrap();
    assert!(
        !response.status().is_success(),
        "Should not allow creating alert without authentication"
    );

    context.stop().await;
}

#[tokio::test]
async fn test_filter_alerts_no_results() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    // Query for an identifier that does not exist
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:3000/api/alert/filter?identifier=doesnotexist")
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "Filter request should succeed"
    );
    let alerts: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(
        alerts.is_empty(),
        "No alerts should be returned for unknown identifier"
    );

    context.stop().await;
}
