use std::net::SocketAddr;

use sqlx::postgres::PgConnectOptions;

/// Specifies the log's output format
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum LogFormat {
    Text,
    TextColor,
    Json,
}

/// Specifies how much log output the app generates
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn tracing_level(&self) -> tracing::Level {
        use tracing::Level;

        match self {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

#[derive(Clone, Debug, clap::Parser)]
#[clap(about, version, propagate_version = true)]
pub struct Settings {
    /// The database URL to connect to. Needs to be a valid libpq
    /// connection URL, like `postgres://postgres@127.0.0.1/nginx_logs`
    #[clap(long, env = "DATABASE_URL")]
    pub database_url: PgConnectOptions,

    /// The maximum size of one INSERT batch to dump into the database. Must be
    /// at least 1
    #[clap(long, env = "INSERT_BATCH_SIZE", default_value = "10")]
    pub insert_batch_size: usize,

    /// To reduce database load, we wait at least this amount of milliseconds
    /// before firing a batched insert query to give the buffer the time to
    /// reach INSERT_BATCH_SIZE. If the buffer is full, however, we ignore this
    /// time limit.
    #[clap(long, env = "INSERT_TIMEOUT", default_value = "1000")]
    pub insert_timeout: u64,

    /// The Socket Address the server should listen on
    #[clap(long, env = "LISTEN_ADDR", default_value = "[::1]:8514")]
    pub listen_addr: SocketAddr,

    /// Defines how the log output will be formatted
    #[clap(value_enum, long, env = "LOG_FORMAT", default_value_t = LogFormat::TextColor)]
    pub log_format: LogFormat,

    /// Defines how noisy the server should be
    #[clap(value_enum, long, env = "LOG_LEVEL", default_value_t = LogLevel::Warn)]
    pub log_level: LogLevel,

    /// Maximum number of messages in the processing queue
    #[clap(long, env = "QUEUE_SIZE", default_value = "50")]
    pub queue_size: usize,

    /// Limits the number of threads used - defaults to the number of CPU cores
    #[clap(long, env = "THREADS")]
    pub threads: Option<usize>,
}
