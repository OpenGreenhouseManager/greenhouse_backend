use greenhouse_core::auth_service_dto::generate_one_time_token::GenerateOneTimeTokenRequestDto;
use test_helper::TestContext;
mod test_helper;

#[tokio::test]
async fn test_login() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    assert!(!token.is_empty(), "Login token should not be empty");
    context.stop().await;
}

#[tokio::test]
async fn test_generate_one_time_token() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    let one_time_token = get_one_time_token(token, String::from("normaluser")).await;

    assert!(
        !one_time_token.is_empty(),
        "One-time token should not be empty"
    );

    context.stop().await;
}

#[tokio::test]
async fn test_register() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;
    let username = String::from("normaluser");
    let password = String::from("normalpassword");

    let one_time_token = get_one_time_token(token, username.clone()).await;

    let user_token = register_user(username.clone(), password.clone(), one_time_token).await;

    assert!(
        !user_token.is_empty(),
        "User token should not be empty after registration"
    );

    context.stop().await;
}

#[tokio::test]
async fn login_with_registered_user() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let username = String::from("normaluser");
    let password = String::from("normalpassword");

    // Register the user first
    let one_time_token =
        get_one_time_token(test_helper::admin_login().await, username.clone()).await;
    let _ = register_user(username.clone(), password.clone(), one_time_token).await;

    // Now login with the registered user
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3000/api/login")
        .json(&greenhouse_core::auth_service_dto::login::LoginRequestDto { username, password })
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success(), "Login should be successful");

    let login_response: greenhouse_core::auth_service_dto::login::LoginResponseDto =
        response.json().await.unwrap();

    assert!(
        !login_response.token.is_empty(),
        "Login token should not be empty"
    );

    context.stop().await;
}

#[tokio::test]
async fn test_login_with_invalid_credentials() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let username = String::from("invaliduser");
    let password = String::from("invalidpassword");

    // Attempt to login with invalid credentials
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3000/api/login")
        .json(&greenhouse_core::auth_service_dto::login::LoginRequestDto { username, password })
        .send()
        .await
        .unwrap();

    assert!(
        !response.status().is_success(),
        "Login should fail with invalid credentials"
    );

    context.stop().await;
}

#[tokio::test]
async fn test_register_with_invalid_one_time_token() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;
    let username = String::from("normaluser");
    let password = String::from("normalpassword");

    // Attempt to register with an invalid one-time token
    let invalid_one_time_token = String::from("invalidtoken");
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3000/api/register")
        .json(
            &greenhouse_core::auth_service_dto::register::RegisterRequestDto {
                username: username.clone(),
                password: password.clone(),
                one_time_token: invalid_one_time_token,
            },
        )
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();

    assert!(
        !response.status().is_success(),
        "Registration should fail with invalid one-time token"
    );

    context.stop().await;
}

#[tokio::test]
async fn test_register_with_existing_username() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;
    let username = String::from("normaluser");
    let password = String::from("normalpassword");

    // Register the user first
    let one_time_token = get_one_time_token(token, username.clone()).await;
    let user_token =
        register_user(username.clone(), password.clone(), one_time_token.clone()).await;

    assert!(
        !user_token.is_empty(),
        "User token should not be empty after registration"
    );

    // Attempt to register again with the same username
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3000/api/register")
        .json(
            &greenhouse_core::auth_service_dto::register::RegisterRequestDto {
                username: username.clone(),
                password: password.clone(),
                one_time_token: one_time_token.clone(),
            },
        )
        .send()
        .await
        .unwrap();

    assert!(
        !response.status().is_success(),
        "Registration should fail with existing username"
    );

    context.stop().await;
}

#[tokio::test]
async fn test_registered_login_with_wrong_password() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let username = String::from("normaluser");
    let password = String::from("normalpassword");

    // Register the user first
    let one_time_token =
        get_one_time_token(test_helper::admin_login().await, username.clone()).await;
    let user_token = register_user(username.clone(), password.clone(), one_time_token).await;

    assert!(
        !user_token.is_empty(),
        "User token should not be empty after registration"
    );

    // Now attempt to login with the registered user but wrong password
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3000/api/login")
        .json(&greenhouse_core::auth_service_dto::login::LoginRequestDto {
            username,
            password: String::from("wrongpassword"),
        })
        .send()
        .await
        .unwrap();

    assert!(
        !response.status().is_success(),
        "Login should fail with wrong password"
    );

    context.stop().await;
}

async fn get_one_time_token(admin_jwt: String, username: String) -> String {
    let client = reqwest::Client::new();
    let result = client
        .post("http://localhost:3000/api/settings/generate_one_time_token")
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={admin_jwt}"))
        .json(&GenerateOneTimeTokenRequestDto { username })
        .send()
        .await
        .unwrap();

    assert!(
        result.status().is_success(),
        "API call failed with status: {}",
        result.status()
    );

    result.json().await.unwrap()
}

async fn register_user(username: String, password: String, one_time_token: String) -> String {
    let client = reqwest::Client::new();
    let result = client
        .post("http://localhost:3000/api/register")
        .json(
            &greenhouse_core::auth_service_dto::register::RegisterRequestDto {
                username,
                password,
                one_time_token,
            },
        )
        .send()
        .await
        .unwrap();

    assert!(
        result.status().is_success(),
        "API call failed with status: {}",
        result.status()
    );

    result
        .json::<greenhouse_core::auth_service_dto::register::RegisterResponseDto>()
        .await
        .unwrap()
        .token
}
