use anyhow::Result;
use clap::Parser;
use sqlx::postgres::PgPoolOptions;

use nginx_syslog_postgres_bridge::{Bridge, Settings};

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::parse();
    tracing_subscriber::fmt::init();

    let udp_socket = tokio::net::UdpSocket::bind(settings.listen_addr).await?;
    let db_pool = PgPoolOptions::new()
        .max_connections(
            num_cpus::get()
                .try_into()
                .expect("number of CPU cores should fit into an u32"),
        )
        .connect_with(settings.database_url)
        .await?;
    sqlx::migrate!().run(&db_pool).await?;

    Bridge::run(db_pool, settings.queue_size, udp_socket).await
}
