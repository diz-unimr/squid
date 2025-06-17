mod api;
mod config;

use crate::config::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // config
    let config = AppConfig::new().expect("Failed to load config");

    // start ssl checker
    tokio::spawn(cert::checker::run());

    // run server
    api::server::serve(config).await?;

    Ok(())
}
