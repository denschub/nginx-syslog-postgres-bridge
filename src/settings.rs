#[derive(clap::Parser, Debug)]
#[clap(author, version, about)]
pub struct Settings {
    #[clap(
        long = "database",
        help = "libpq-compatible postgres:// connection URI",
        env = "NGINXPG_DATABASE",
        default_value = "postgres://postgres@postgres:5432/nginx_logs"
    )]
    pub database_uri: String,

    #[clap(
        long = "listen",
        help = "Listen Address for this server",
        env = "NGINXPG_LISTEN",
        default_value = "[::]:514"
    )]
    pub listen_addr: String,

    #[clap(
        long = "queue-size",
        help = "Maximum number of messages in the processing queue",
        env = "NGINXPG_QUEUE_SIZE",
        default_value = "10000"
    )]
    pub queue_size: usize,
}
