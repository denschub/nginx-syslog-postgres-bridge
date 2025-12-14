use sqlx::{PgPool, postgres::PgConnectOptions};
use std::{str::FromStr, time::Duration};
use tokio::time::sleep;

use nginx_syslog_postgres_bridge::{Bridge, Settings};

pub async fn spawn_test_server(db_pool: PgPool) -> String {
    let settings = Settings {
        database_url: PgConnectOptions::new(),
        insert_batch_size: 1,
        insert_timeout: 100,
        listen_addr: std::net::SocketAddr::from_str("127.0.0.1:0").unwrap(),
        queue_size: 100,
        threads: None,
    };

    let socket = tokio::net::UdpSocket::bind(settings.listen_addr)
        .await
        .unwrap();
    let listening_port = socket.local_addr().unwrap().port();

    tokio::spawn(Bridge::run(db_pool, settings, socket));

    format!("127.0.0.1:{}", listening_port)
}

pub async fn send_datagram(bytes: &[u8], destination: String) {
    let socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    socket.connect(destination).await.unwrap();
    socket.send(bytes).await.unwrap();
}

pub async fn wait_for_insert() {
    // [ToDo] Sooooo... this is kinda bad. However, since I store the data
    // asynchronously, I'd need to some wait to actually figure out when the
    // data is written into the database...
    sleep(Duration::from_millis(1000)).await;
}
