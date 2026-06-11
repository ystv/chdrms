use chdrms_server::{error::AppError, state::AppState};
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::TraceLayer;
use tracing_subscriber::prelude::*;

macro_rules! get_env {
    ($name:expr) => {
        std::env::var($name).expect(concat!("environment variable ", $name, " to be set"))
    };
    ($name:expr, $default:expr) => {
        std::env::var($name).unwrap_or_else(|_| $default.to_string())
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

    let config =
        chdrms_server::config::load_configuration(get_env!("CONFIGURATION_PATH", "config.toml"))
            .await;

    let key = chdrms_server::config::load_key(get_env!("SECRET_KEY_PATH", ".secretkey")).await;

    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&get_env!("DATABASE_URL"))
        .await?;

    if let Err(e) = chdrms_database::migrate(&pool).await {
        tracing::error!(?e, "failed to migrate database");
        return Ok(());
    };

    let state = AppState::new(pool, config, key);

    let app = chdrms_server::routes::routes(state).layer(TraceLayer::new_for_http());

    let host = get_env!("HOST", "::");

    let port: u16 = get_env!("PORT", "3000").parse().expect("valid port number");

    let listener = tokio::net::TcpListener::bind((host, port)).await.unwrap();

    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
