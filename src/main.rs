use anyhow::Result;
use clap::Parser;

use nginx_syslog_postgres_bridge::Bridge;
use nginx_syslog_postgres_bridge::Settings;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let settings = Settings::parse();

    let bridge = Bridge::build(settings).await?;
    bridge.run().await
}
