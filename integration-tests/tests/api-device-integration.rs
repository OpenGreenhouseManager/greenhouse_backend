use greenhouse_core::device_service_dto::{
    get_device::DeviceResponseDto, post_device::PostDeviceDtoRequest,
};
use test_helper::TestContext;
mod test_helper;

#[tokio::test]
async fn test_create_device_entry() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    // Create diary entry
    let client = reqwest::Client::new();
    let post_entry = PostDeviceDtoRequest {
        address: String::from("0.0.0.0:80"),
        can_script: true,
        name: String::from("testDevice"),
        description: String::from("test Description"),
    };
    let response = client
        .post("http://localhost:3000/api/device")
        .json(&post_entry)
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "Failed to create diary entry"
    );

    let device: DeviceResponseDto = response.json().await.unwrap();

    assert_eq!(device.address, String::from("0.0.0.0:80"),);
    assert!(device.canscript);
    assert_eq!(device.description, String::from("testDevice"),);
    assert_eq!(device.name, String::from("test Description"));

    context.stop().await;
}

#[tokio::test]
async fn test_create_and_get_device_entry() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let _token = test_helper::admin_login().await;

    context.stop().await;
}

#[tokio::test]
async fn test_create_and_update_device_entry() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let _token = test_helper::admin_login().await;

    context.stop().await;
}

#[tokio::test]
async fn test_status_for_not_existing_device_entry() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let _token = test_helper::admin_login().await;

    context.stop().await;
}

#[tokio::test]
async fn test_status_for_offline_device_entry() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let _token = test_helper::admin_login().await;

    context.stop().await;
}
