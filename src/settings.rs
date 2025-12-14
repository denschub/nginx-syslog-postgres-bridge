use std::net::SocketAddr;

use sqlx::postgres::PgConnectOptions;

#[derive(Clone, Debug, clap::Parser)]
#[clap(about, version, propagate_version = true)]
pub struct Settings {
    /// The database URL to connect to. Needs to be a valid libpq
    /// connection URL, like `postgres://postgres@127.0.0.1/nginx_logs`
    #[clap(long, short, env = "DATABASE_URL")]
    pub database_url: PgConnectOptions,

    /// The maximum size of one INSERT batch to dump into the database. Must be
    /// at least 1
    #[clap(long, short('b'), env = "INSERT_BATCH_SIZE", default_value = "10")]
    pub insert_batch_size: usize,

    /// To reduce database load, we wait at least this amount of milliseconds
    /// before firing a batched insert query to give the buffer the time to
    /// reach INSERT_BATCH_SIZE. If the buffer is full, however, we ignore this
    /// time limit.
    #[clap(long, short, env = "INSERT_TIMEOUT", default_value = "250")]
    pub insert_timeout: u64,

    /// The Socket Address the server should listen on
    #[clap(long, short, env = "LISTEN_ADDR", default_value = "[::1]:8514")]
    pub listen_addr: SocketAddr,

    /// Maximum number of messages in the processing queue
    #[clap(long, short, env = "QUEUE_SIZE", default_value = "10000")]
    pub queue_size: usize,

    /// Limits the number of threads used - defaults to the number of CPU cores
    #[clap(long, short, env = "THREADS")]
    pub threads: Option<usize>,
}
