use std::net::SocketAddr;

use sqlx::postgres::PgConnectOptions;

#[derive(Debug, clap::Parser)]
#[clap(about, version, propagate_version = true)]
pub struct Settings {
    /// The database URL to connect to. Needs to be a valid libpq
    /// connection URL, like `postgres://postgres@127.0.0.1/nginx_logs`
    #[clap(long, short, env = "DATABASE_URL")]
    pub database_url: PgConnectOptions,

    /// The Socket Address the server should listen on
    #[clap(long, short, env = "LISTEN_ADDR", default_value = "[::1]:8514")]
    pub listen_addr: SocketAddr,

    /// Maximum number of messages in the processing queue
    #[clap(long, short, env = "QUEUE_SIZE", default_value = "10000")]
    pub queue_size: usize,

    /// Limits the number of threads used - defaults to the number of CPU cores
    #[clap(long, env = "THREADS")]
    pub threads: Option<usize>,
}
