use std::path::PathBuf;

use chdrms_server::{error::AppError, state::AppState, storage};
use sqlx::postgres::PgPoolOptions;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::prelude::*;

macro_rules! get_env {
    ($name:expr) => {
        std::env::var($name).expect(concat!("environment variable ", $name, " to be set"))
    };
    ($name:expr, $default:expr) => {
        std::env::var($name)
            .ok()
            .map(|val| {
                val.parse()
                    .unwrap_or_else(|err| panic!("variable '{}' wrong format: {:?}", $name, err))
            })
            .unwrap_or_else(|| $default)
    };
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("info,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = chdrms_server::config::load_configuration(get_env!(
        "CONFIGURATION_PATH",
        "config.toml".to_string()
    ))
    .await;

    let key =
        chdrms_server::config::load_key(get_env!("SECRET_KEY_PATH", ".secretkey".to_string()))
            .await;

    // database
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&get_env!("DATABASE_URL"))
        .await?;

    if let Err(e) = chdrms_database::migrate(&pool).await {
        tracing::error!(?e, "failed to migrate database");
        return Ok(());
    };

    // storage
    let storage = storage::Storage::new(&config.storage);

    let state = AppState::new(pool, config, storage, key);

    let (app, _) = chdrms_server::routes::routes();

    let mut app = app.with_state(state);

    // if we're running in production, we want to serve the static files from `dist`,
    // otherwise, we let Vite proxy the requests to us.
    if get_env!("ENVIRONMENT", "DEVELOPMENT".to_string()).as_str() == "PRODUCTION" {
        let directory: PathBuf = get_env!("UI_DIRECTORY", "dist".to_string()).into();
        app = app.fallback_service(
            ServeDir::new(&directory)
                .not_found_service(ServeFile::new(directory.join("index.html"))),
        )
    }

    app = app.layer(TraceLayer::new_for_http());

    let host: String = get_env!("HOST", "::".to_string());
    let port: u16 = get_env!("PORT", 3000);

    let listener = tokio::net::TcpListener::bind((host, port)).await.unwrap();

    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
