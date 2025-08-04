use greenhouse_core::device_service_dto::{
    get_device::DeviceResponseDto, post_device::PostDeviceDtoRequest,
    put_device::PutDeviceDtoRequest,
};
use test_helper::TestContext;
mod test_helper;

#[tokio::test]
async fn test_create_device_entry() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    // Create device entry
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
        "Failed to create device entry with error: {}",
        response.text().await.unwrap()  
    );

    let device: DeviceResponseDto = response.json().await.unwrap();

    assert_eq!(device.address, String::from("0.0.0.0:80"));
    assert!(device.canscript);
    assert_eq!(device.name, String::from("testDevice"));
    assert_eq!(device.description, String::from("test Description"));

    context.stop().await;
}

#[tokio::test]
async fn test_create_and_get_device_entry() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    let client = reqwest::Client::new();

    // Create device entry
    let post_entry = PostDeviceDtoRequest {
        address: String::from("192.168.1.100:8080"),
        can_script: false,
        name: String::from("TestDevice2"),
        description: String::from("Second test device"),
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
        "Failed to create device entry with error: {}",
        response.text().await.unwrap()  
    );

    let created_device: DeviceResponseDto = response.json().await.unwrap();

    // Get device by ID
    let response = client
        .get(format!(
            "http://localhost:3000/api/device/{}",
            created_device.id
        ))
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success(), "Failed to get device entry");

    let retrieved_device: DeviceResponseDto = response.json().await.unwrap();

    // Verify the retrieved device matches what we created
    assert_eq!(retrieved_device.id, created_device.id);
    assert_eq!(retrieved_device.name, "TestDevice2");
    assert_eq!(retrieved_device.description, "Second test device");
    assert_eq!(retrieved_device.address, "192.168.1.100:8080");
    assert!(!retrieved_device.canscript);

    context.stop().await;
}

#[tokio::test]
async fn test_create_and_update_device_entry() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    let client = reqwest::Client::new();

    // Create device entry
    let post_entry = PostDeviceDtoRequest {
        address: String::from("10.0.0.1:3000"),
        can_script: true,
        name: String::from("OriginalDevice"),
        description: String::from("Original description"),
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
        "Failed to create device entry with error: {}",
        response.text().await.unwrap()  
    );

    let created_device: DeviceResponseDto = response.json().await.unwrap();

    // Update the device
    let put_entry = PutDeviceDtoRequest {
        address: String::from("10.0.0.2:4000"),
        can_script: false,
        name: String::from("UpdatedDevice"),
        description: String::from("Updated description"),
    };

    let response = client
        .put(format!(
            "http://localhost:3000/api/device/{}",
            created_device.id
        ))
        .json(&put_entry)
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "Failed to update device entry"
    );

    let updated_device: DeviceResponseDto = response.json().await.unwrap();

    // Verify the update was successful
    assert_eq!(updated_device.id, created_device.id);
    assert_eq!(updated_device.name, "UpdatedDevice");
    assert_eq!(updated_device.description, "Updated description");
    assert_eq!(updated_device.address, "10.0.0.2:4000");
    assert!(!updated_device.canscript);

    context.stop().await;
}

#[tokio::test]
async fn test_status_for_not_existing_device_entry() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    let client = reqwest::Client::new();
    let non_existent_id = uuid::Uuid::new_v4();

    // Try to get status for a device that doesn't exist
    let response = client
        .get(format!(
            "http://localhost:3000/api/device/{non_existent_id}/status"
        ))
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();

    assert!(
        response.status().is_client_error(),
        "Should return client error for non-existent device"
    );

    context.stop().await;
}

#[tokio::test]
async fn test_status_for_offline_device_entry() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    let client = reqwest::Client::new();

    // Create device with unreachable address
    let post_entry = PostDeviceDtoRequest {
        address: String::from("192.168.999.999:8080"), // Invalid IP
        can_script: true,
        name: String::from("OfflineDevice"),
        description: String::from("Device that can't be reached"),
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
        "Failed to create device entry with error: {}",
        response.text().await.unwrap()  
    );

    let created_device: DeviceResponseDto = response.json().await.unwrap();

    // Try to get status for the offline device
    let response = client
        .get(format!(
            "http://localhost:3000/api/device/{}/status",
            created_device.id
        ))
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();

    // Should return an error status (likely 500 or 503) since device is unreachable
    assert!(
        response.status().is_server_error() || response.status().is_client_error(),
        "Should return error status for offline device with error: {}",
        response.text().await.unwrap()  
    );

    context.stop().await;
}

#[tokio::test]
async fn test_get_all_devices() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    let client = reqwest::Client::new();

    // Create multiple devices
    let devices_to_create = vec![
        PostDeviceDtoRequest {
            address: String::from("10.0.1.1:8000"),
            can_script: true,
            name: String::from("Device1"),
            description: String::from("First device"),
        },
        PostDeviceDtoRequest {
            address: String::from("10.0.1.2:8000"),
            can_script: false,
            name: String::from("Device2"),
            description: String::from("Second device"),
        },
    ];

    for device in devices_to_create {
        let response = client
            .post("http://localhost:3000/api/device")
            .json(&device)
            .header("Access-Control-Allow-Credentials", "true")
            .header("Cookie", format!("auth-token={token}"))
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success(), "Failed to create device with error: {}",
        response.text().await.unwrap()  
    );
    }

    // Get all devices
    let response = client
        .get("http://localhost:3000/api/device")
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success(), "Failed to get all devices with error: {}",
        response.text().await.unwrap()  
    );

    let devices: Vec<DeviceResponseDto> = response.json().await.unwrap();
    assert!(devices.len() >= 2, "Should have at least 2 devices");

    // Verify we can find our created devices
    let device_names: Vec<&String> = devices.iter().map(|d| &d.name).collect();
    assert!(device_names.contains(&&String::from("Device1")));
    assert!(device_names.contains(&&String::from("Device2")));

    context.stop().await;
}

#[tokio::test]
async fn test_get_device_config() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    let client = reqwest::Client::new();

    // Create device
    let post_entry = PostDeviceDtoRequest {
        address: String::from("192.168.999.999:8080"), // Invalid IP to test error handling
        can_script: true,
        name: String::from("ConfigTestDevice"),
        description: String::from("Device for config testing"),
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
        "Failed to create device entry with error: {}",
        response.text().await.unwrap()
    );

    let created_device: DeviceResponseDto = response.json().await.unwrap();

    // Try to get config for the device (this will likely fail since the device is unreachable)
    let response = client
        .get(format!(
            "http://localhost:3000/api/device/{}/config",
            created_device.id
        ))
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();

    // Since the device is unreachable, this should return an error
    // In a real scenario with reachable devices, this would return 200 with config data
    assert!(
        response.status().is_server_error() || response.status().is_client_error(),
        "Should return error status for unreachable device config request with error: {}",
        response.text().await.unwrap()  
    );

    context.stop().await;
}
