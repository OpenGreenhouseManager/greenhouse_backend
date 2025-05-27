use test_helper::TestContext;
mod test_helper;

#[tokio::test]
async fn test_api_to_microservice_integration() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::login().await;

    let client = reqwest::Client::new();
    let result = client
        .get("http://localhost:3000/api/test")
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();

    assert!(
        result.status().is_success(),
        "API call failed with status: {}",
        result.status()
    );
    context.stop().await;
}
