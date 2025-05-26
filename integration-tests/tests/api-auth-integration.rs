use test_helper::start_all_services;

mod test_helper;

#[tokio::test]
async fn test_api_to_microservice_integration() {
    start_all_services().await;

    let client = reqwest::Client::new();
    let result = client
        .get("http://localhost:3000/api/test2")
        .send()
        .await
        .unwrap();
    assert!(
        result.status().is_success(),
        "API call failed with status: {}",
        result.status()
    );
}
