use anyhow::{Error, Result};
use sqlx::PgPool;
use tokio::{
    net::UdpSocket,
    sync::mpsc::{Receiver, Sender, channel},
};
use tracing::{info, trace};

use crate::parsers::AccessLogEntry;

pub struct Bridge {}

impl Bridge {
    pub async fn run(db_pool: PgPool, queue_size: usize, udp_socket: UdpSocket) -> Result<()> {
        let (tx, rx) = channel::<AccessLogEntry>(queue_size);

        let udp_receiver = UdpReceiver::new(tx, udp_socket);
        let receiving_loop = tokio::spawn(async move { udp_receiver.run().await });

        let mut queue_item_storer = QueueItemStorer::new(db_pool, rx);
        let storing_loop = tokio::spawn(async move { queue_item_storer.run().await });

        tokio::select! {
            _ = receiving_loop => {},
            _ = storing_loop => {},
        };

        Ok(())
    }
}

pub struct UdpReceiver {
    received_sender: Sender<AccessLogEntry>,
    socket: UdpSocket,
}

impl UdpReceiver {
    pub fn new(received_sender: Sender<AccessLogEntry>, socket: UdpSocket) -> Self {
        Self {
            received_sender,
            socket,
        }
    }

    pub async fn run(&self) {
        // As per RFC5426, a syslog-via-udp message can only ever be one UDP
        // datagram long, not more. So we know the maximum ever length of that,
        // and the size is small enough to just allocate everything.
        let mut buf = [0; 65535];

        loop {
            if let Ok((len, addr)) = self.socket.recv_from(&mut buf).await {
                info!("Received {} bytes from {}", len, addr);

                let buf = buf[0..len].to_owned();
                let tx_clone = self.received_sender.clone();
                tokio::spawn(async move {
                    if let Ok(line) = std::str::from_utf8(&buf) {
                        trace!("Raw message: `{}`", line);
                        if let Ok(entry) = Self::parse_datagram(line).await {
                            // Silently drop send errors. This will fail if
                            // There's too much traffic, but if that's the case,
                            // spamming things to STDOUT doesn't help.
                            let _ = tx_clone.try_send(entry);
                        }
                    }
                });
            }
        }
    }

    async fn parse_datagram(datagram: &str) -> Result<AccessLogEntry> {
        // at the moment, I'm completely ignoring everything provided by syslog
        // except the message. I could skip the syslog parsing, and just look for
        // the opening {, then read from there.
        // However, in the future, I might expand this with the ability to handle
        // error_log as well... so let's keep this for now.
        let syslog = syslog_loose::parse_message(datagram, syslog_loose::Variant::Either);
        serde_json::from_str(syslog.msg).map_err(Error::msg)
    }
}

struct QueueItemStorer {
    db_pool: PgPool,
    receiver: Receiver<AccessLogEntry>,
}

impl QueueItemStorer {
    pub fn new(db_pool: PgPool, receiver: Receiver<AccessLogEntry>) -> Self {
        Self { db_pool, receiver }
    }

    pub async fn store_single(&self, entry: &AccessLogEntry) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO access_log (
                id,
                event_ts,
                hostname,
                server_name,
                server_port,
                client_addr,
                client_forwarded_for,
                client_referer,
                client_ua,
                req_host,
                req_length,
                req_method,
                req_proto,
                req_scheme,
                req_uri,
                res_body_length,
                res_duration,
                res_length,
                res_status,
                upstream_addr,
                upstream_bytes_received,
                upstream_bytes_sent,
                upstream_cache_status,
                upstream_connect_time,
                upstream_host,
                upstream_response_length,
                upstream_response_time,
                upstream_status
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
                $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28
            )
            "#,
            uuid::Uuid::new_v4(),
            entry.ts,
            entry.hostname,
            entry.server.name,
            entry.server.port,
            entry.client.addr,
            entry.client.forwarded_for,
            entry.client.referer,
            entry.client.ua,
            entry.req.host,
            entry.req.length,
            entry.req.method,
            entry.req.proto,
            entry.req.scheme,
            entry.req.uri,
            entry.res.body_length,
            entry.res.duration,
            entry.res.length,
            entry.res.status,
            entry.upstream.addr,
            entry.upstream.bytes_received,
            entry.upstream.bytes_sent,
            entry.upstream.cache_status,
            entry.upstream.connect_time,
            entry.upstream.host,
            entry.upstream.response_length,
            entry.upstream.response_time,
            entry.upstream.status
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(entry) = self.receiver.recv().await {
                let _ = self.store_single(&entry).await;
            }
        }
    }
}
