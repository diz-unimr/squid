use crate::api::cert;
use crate::config::AppConfig;
use axum::{Router, routing::get};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub(crate) struct ApiContext {
    pub(crate) db: DatabaseConnection,
}

pub async fn serve(config: AppConfig) -> anyhow::Result<()> {
    let filter = format!(
        "{}={level},tower_http={level}",
        env!("CARGO_CRATE_NAME"),
        level = config.app.log_level
    );
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into()))
        // .with_span_events(FmtSpan::CLOSE)
        .init();

    let mut db_opts = ConnectOptions::new("sqlite://db.sqlite?mode=rwc");
    db_opts
        .max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info);

    let db = Database::connect(db_opts).await?;
    Migrator::up(&db, None).await?;

    let state = Arc::new(ApiContext { db });
    let router = api_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, router).await.map_err(|e| e.into())
}

async fn root() -> &'static str {
    "Squid Web API"
}

fn api_router(state: Arc<ApiContext>) -> Router {
    Router::new()
        .route("/", get(root))
        .merge(cert::router())
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}
