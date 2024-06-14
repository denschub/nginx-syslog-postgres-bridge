use anyhow::Result;
use clap::Parser;
use sqlx::postgres::PgPoolOptions;

use nginx_syslog_postgres_bridge::{Bridge, Settings};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let settings = Settings::parse();

    let udp_socket = tokio::net::UdpSocket::bind(settings.listen_addr).await?;
    let db_pool = PgPoolOptions::new()
        .connect_with(settings.database_url)
        .await?;
    sqlx::migrate!().run(&db_pool).await?;

    Bridge::run(db_pool, settings.queue_size, udp_socket).await
}
