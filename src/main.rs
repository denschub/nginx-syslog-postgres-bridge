use anyhow::Result;
use clap::Parser;
use sqlx::postgres::PgPoolOptions;

use nginx_syslog_postgres_bridge::{Bridge, Settings};

fn main() -> Result<()> {
    let settings = Settings::parse();

    let mut rt = tokio::runtime::Builder::new_multi_thread();
    if let Some(threads) = settings.threads {
        rt.worker_threads(threads);
    }

    rt.enable_all()
        .build()?
        .block_on(async { run(settings).await })
}

async fn run(settings: Settings) -> Result<()> {
    tracing_subscriber::fmt::init();

    let udp_socket = tokio::net::UdpSocket::bind(settings.listen_addr).await?;
    let db_pool = PgPoolOptions::new()
        .max_connections(
            tokio::runtime::Handle::current()
                .metrics()
                .num_workers()
                .try_into()
                .expect("num_workers to be less than 2^32"),
        )
        .connect_with(settings.database_url)
        .await?;
    sqlx::migrate!().run(&db_pool).await?;

    Bridge::run(db_pool, settings.queue_size, udp_socket).await
}
