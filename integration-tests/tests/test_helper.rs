use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use once_cell::sync::Lazy;
use testcontainers::{GenericImage, ImageExt, core::IntoContainerPort, runners::AsyncRunner};

const AUTH_POSTGRES_URL: &str = "postgres://postgres:password@localhost:5432";
const DATA_STORAGE_POSTGRES_URL: &str = "postgres://postgres:password@localhost:5433";

const AUTH_SERVICE_CONFIG: Lazy<auth_service::Config> = Lazy::new(|| auth_service::Config {
    service_port: 3001,
    database_url: format!("{}/auth_service", AUTH_POSTGRES_URL),
    jwt_secret: String::from("supersecretkey"),
    sentry_url: String::from("http://localhost:9000"),
});
const DATA_STORAGE_SERVICE_CONFIG: Lazy<data_storage_service::Config> =
    Lazy::new(|| data_storage_service::Config {
        service_port: 3002,
        database_url: format!("{}/data_storage_service", DATA_STORAGE_POSTGRES_URL),
        sentry_url: String::from("http://localhost:9000"),
    });
const WEB_API_CONFIG: Lazy<web_api::Config> = Lazy::new(|| web_api::Config {
    api_port: 3000,
    service_addresses: web_api::ServiceAddresses {
        auth_service: String::from("http://localhost:3001"),
        data_storage_service: String::from("http://localhost:3002"),
    },
    sentry_url: String::from("http://localhost:9000"),
});

const POSTGRES_VERSION: &str = "15";
pub async fn start_all_services() {
    start_postgres().await;

    start_auth_service().await;
    start_data_storage_service().await;
    start_web_api().await;
}

async fn start_postgres() {
    GenericImage::new("postgres", POSTGRES_VERSION)
        .with_exposed_port(5432.tcp())
        .with_network("bridge")
        .with_env_var("DEBUG", "1")
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "password")
        .with_env_var("POSTGRES_DB", "auth")
        .start()
        .await
        .unwrap();
    GenericImage::new("postgres", POSTGRES_VERSION)
        .with_exposed_port(5433.tcp())
        .with_network("bridge")
        .with_env_var("DEBUG", "1")
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "password")
        .with_env_var("POSTGRES_DB", "data")
        .start()
        .await
        .unwrap();
}

async fn start_auth_service() -> tokio::task::JoinHandle<Result<(), std::io::Error>> {
    let auth_pool = auth_service::Pool::builder()
        .build(AsyncDieselConnectionManager::new(
            AUTH_SERVICE_CONFIG.database_url.clone(),
        ))
        .await
        .unwrap();
    let auth_service_app = auth_service::app(AUTH_SERVICE_CONFIG.clone(), auth_pool.clone());

    let url = format!("0.0.0.0:{}", AUTH_SERVICE_CONFIG.service_port);
    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
    tokio::spawn(async move { axum::serve(listener, auth_service_app).await })
}

async fn start_data_storage_service() -> tokio::task::JoinHandle<Result<(), std::io::Error>> {
    let data_storage_pool = data_storage_service::Pool::builder()
        .build(AsyncDieselConnectionManager::new(
            DATA_STORAGE_SERVICE_CONFIG.database_url.clone(),
        ))
        .await
        .unwrap();
    let data_storage_service_app = data_storage_service::app(
        DATA_STORAGE_SERVICE_CONFIG.clone(),
        data_storage_pool.clone(),
    );

    let url = format!("0.0.0.0:{}", DATA_STORAGE_SERVICE_CONFIG.service_port);
    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
    tokio::spawn(async move { axum::serve(listener, data_storage_service_app).await })
}

async fn start_web_api() -> tokio::task::JoinHandle<Result<(), std::io::Error>> {
    let api_app = web_api::app(WEB_API_CONFIG.clone());
    let url = format!("0.0.0.0:{}", WEB_API_CONFIG.api_port);
    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
    tokio::spawn(async move { axum::serve(listener, api_app).await })
}
