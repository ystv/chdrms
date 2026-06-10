use chdrms_server::{error::AppError, state::AppState};
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::TraceLayer;
use tracing_subscriber::prelude::*;

macro_rules! get_env {
    ($name:expr) => {
        std::env::var($name).expect(concat!("environment variable ", $name, " to be set"))
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

    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&get_env!("DATABASE_URL"))
        .await?;

    if let Err(e) = sqlx::migrate!("../database/migrations").run(&pool).await {
        tracing::error!(?e, "failed to migrate database");
        return Ok(());
    };

    let state = AppState::new(pool);

    let app = chdrms_server::routes::routes(state).layer(TraceLayer::new_for_http());

    let host = std::env::var("HOST").unwrap_or_else(|_| "[::]".to_string());

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("valid port number");

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();

    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
