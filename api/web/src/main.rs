use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use web_api::Config;

fn main() {
    let config = load_config();

    let _guard = sentry::init((
        config.sentry_url.clone(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                        "web_api=debug,tower_http=debug,axum::rejection=trace".into()
                    }),
                )
                .with(tracing_subscriber::fmt::layer())
                .init();

            // build our application with a route
            let url = format!("0.0.0.0:{}", config.api_port);

            let app = web_api::app(config);

            // run it
            let listener = tokio::net::TcpListener::bind(url).await.unwrap();

            tracing::info!("listening on {}", listener.local_addr().unwrap());
            axum::serve(listener, app).await.unwrap();
        });
}

fn load_config() -> Config {
    let file_path = if cfg!(debug_assertions) {
        "api/web/config/.env"
    } else {
        "config/.env"
    };

    match std::fs::File::open(file_path) {
        Ok(f) => match serde_yaml::from_reader(f) {
            Ok(config) => config,
            Err(e) => {
                panic!("Failed to read config file: {e}")
            }
        },
        Err(_) => {
            panic!("Failed to open config file at: {file_path}")
        }
    }
}
