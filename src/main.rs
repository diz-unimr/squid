mod api;
mod config;

use crate::config::AppConfig;
use cert::checker::Checker;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // config
    let config = AppConfig::new().expect("Failed to load config");

    // start ssl checker
    let checker = Checker::new();
    tokio::spawn(async move {
        checker.run().await;
    });

    // run server
    api::server::serve(config).await?;

    Ok(())
}
