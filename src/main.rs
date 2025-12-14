use anyhow::{Result, bail};
use clap::Parser;
use sqlx::postgres::PgPoolOptions;

use nginx_syslog_postgres_bridge::{Bridge, settings::LogFormat, settings::Settings};

fn main() -> Result<()> {
    let settings = Settings::parse();

    if settings.insert_batch_size < 1 {
        bail!("INSERT_BATCH_SIZE must be at least 1!");
    }

    let mut rt = tokio::runtime::Builder::new_multi_thread();
    if let Some(threads) = settings.threads {
        rt.worker_threads(threads);
    }

    rt.enable_all()
        .build()?
        .block_on(async { run(settings).await })
}

async fn run(settings: Settings) -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(settings.log_level.tracing_level())
        .with_target(false);
    match settings.log_format {
        LogFormat::Text => subscriber.with_ansi(false).init(),
        LogFormat::TextColor => subscriber.with_ansi(true).init(),
        LogFormat::Json => subscriber.json().with_span_list(false).init(),
    }

    let settings_clone = settings.clone();
    let udp_socket = tokio::net::UdpSocket::bind(settings_clone.listen_addr).await?;
    let db_pool = PgPoolOptions::new()
        .max_connections(
            tokio::runtime::Handle::current()
                .metrics()
                .num_workers()
                .try_into()
                .expect("num_workers to be less than 2^32"),
        )
        .connect_with(settings_clone.database_url)
        .await?;
    sqlx::migrate!().run(&db_pool).await?;

    Bridge::run(db_pool, settings, udp_socket).await
}
