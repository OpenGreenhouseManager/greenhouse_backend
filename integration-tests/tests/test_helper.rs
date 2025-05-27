use chrono::Utc;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use greenhouse_core::auth_service_dto::register_admin::RegisterAdminRequestDto;
use greenhouse_core::auth_service_dto::user_token::UserToken;
use jsonwebtoken::{EncodingKey, Header};
use std::hash::{DefaultHasher, Hash, Hasher};
use testcontainers::{ContainerAsync, ImageExt};
use testcontainers_modules::postgres::{self, Postgres};
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use tokio::task::JoinHandle;

pub(crate) const TEST_USERNAME: &str = "testuser";
pub(crate) const TEST_PASSWORD: &str = "testpassword";
pub(crate) const AUTH_SECRET: &str = "testpassword";

pub(crate) struct TestContext {
    auth_postgres_container: Option<ContainerAsync<Postgres>>,
    data_storage_postgres_container: Option<ContainerAsync<Postgres>>,
    auth_service: Option<JoinHandle<Result<(), std::io::Error>>>,
    data_storage_service: Option<JoinHandle<Result<(), std::io::Error>>>,
    web_api: Option<JoinHandle<Result<(), std::io::Error>>>,
}

impl TestContext {
    pub(crate) fn new() -> Self {
        Self {
            auth_postgres_container: None,
            data_storage_postgres_container: None,
            auth_service: None,
            data_storage_service: None,
            web_api: None,
        }
    }

    pub(crate) async fn start_all_services(&mut self) {
        if self.data_storage_postgres_container.is_none() {
            self.data_storage_postgres_container = Some(start_data_storage_postgres().await);
        }
        let data_port = self
            .data_storage_postgres_container
            .as_ref()
            .unwrap()
            .get_host_port_ipv4(5432)
            .await
            .unwrap();

        if self.auth_postgres_container.is_none() {
            self.auth_postgres_container = Some(start_auth_postgres().await);
        }
        let auth_port = self
            .auth_postgres_container
            .as_ref()
            .unwrap()
            .get_host_port_ipv4(5432)
            .await
            .unwrap();

        if self.auth_service.is_none() {
            self.auth_service = Some(
                start_auth_service(format!(
                    "postgres://postgres:postgres@localhost:{auth_port}/auth"
                ))
                .await,
            );
        }
        if self.data_storage_service.is_none() {
            self.data_storage_service = Some(
                start_data_storage_service(format!(
                    "postgres://postgres:postgres@localhost:{data_port}/data"
                ))
                .await,
            );
        }
        if self.web_api.is_none() {
            self.web_api = Some(start_web_api().await);
        }
    }

    pub(crate) async fn stop(&self) {
        if let Some(container) = &self.auth_postgres_container {
            container.stop().await.unwrap();
        }
        if let Some(container) = &self.data_storage_postgres_container {
            container.stop().await.unwrap();
        }
        if let Some(auth_service) = &self.auth_service {
            auth_service.abort();
        }
        if let Some(data_storage_service) = &self.data_storage_service {
            data_storage_service.abort();
        }
        if let Some(web_api) = &self.web_api {
            web_api.abort();
        }
    }
}

async fn start_auth_postgres() -> ContainerAsync<Postgres> {
    postgres::Postgres::default()
        .with_db_name("auth")
        .with_tag("latest")
        .start()
        .await
        .unwrap()
}

async fn start_data_storage_postgres() -> ContainerAsync<Postgres> {
    postgres::Postgres::default()
        .with_db_name("data")
        .with_tag("latest")
        .start()
        .await
        .unwrap()
}

async fn start_auth_service(db_url: String) -> tokio::task::JoinHandle<Result<(), std::io::Error>> {
    let auth_service_config = auth_service::Config {
        database_url: db_url,
        service_port: 3001,
        sentry_url: String::new(),
        jwt_secret: AUTH_SECRET.to_string(),
    };

    let auth_pool = auth_service::Pool::builder()
        .build(AsyncDieselConnectionManager::new(
            auth_service_config.database_url.clone(),
        ))
        .await
        .unwrap();
    let auth_service_app = auth_service::app(auth_service_config.clone(), auth_pool.clone());

    let url = format!("0.0.0.0:{}", auth_service_config.service_port);
    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
    tokio::spawn(async move { axum::serve(listener, auth_service_app).await })
}

async fn start_data_storage_service(
    db_url: String,
) -> tokio::task::JoinHandle<Result<(), std::io::Error>> {
    let data_storage_service_config = data_storage_service::Config {
        database_url: db_url,
        service_port: 3002,
        sentry_url: String::new(),
    };

    let data_storage_pool = data_storage_service::Pool::builder()
        .build(AsyncDieselConnectionManager::new(
            data_storage_service_config.database_url.clone(),
        ))
        .await
        .unwrap();
    let data_storage_service_app = data_storage_service::app(
        data_storage_service_config.clone(),
        data_storage_pool.clone(),
    );

    let url = format!("0.0.0.0:{}", data_storage_service_config.service_port);
    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
    tokio::spawn(async move { axum::serve(listener, data_storage_service_app).await })
}

async fn start_web_api() -> tokio::task::JoinHandle<Result<(), std::io::Error>> {
    let web_api_config = web_api::Config {
        api_port: 3000,
        service_addresses: web_api::ServiceAddresses {
            auth_service: String::from("http://localhost:3001"),
            data_storage_service: String::from("http://localhost:3002"),
        },
        sentry_url: String::new(),
    };

    let api_app = web_api::app(web_api_config.clone());
    let url = format!("0.0.0.0:{}", web_api_config.api_port);
    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
    tokio::spawn(async move { axum::serve(listener, api_app).await })
}

pub(crate) fn generate_one_time_token(user_name: &str, secret: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    let str = String::from(user_name) + secret;
    str.hash(&mut hasher);
    hasher.finish()
}

pub(crate) async fn login() -> String {
    register_admin().await;

    api_login().await
}

pub(crate) async fn register_admin() {
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3001/registerAdmin")
        .json(&RegisterAdminRequestDto {
            username: TEST_USERNAME.to_string(),
            password: TEST_PASSWORD.to_string(),
        })
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success(), "Failed to register admin");
}

pub(crate) async fn api_login() -> String {
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3000/api/login")
        .json(&greenhouse_core::auth_service_dto::login::LoginRequestDto {
            username: TEST_USERNAME.to_string(),
            password: TEST_PASSWORD.to_string(),
        })
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success(), "Failed to login");
    let user_token: greenhouse_core::auth_service_dto::login::LoginResponseDto =
        response.json().await.unwrap();
    user_token.token
}
