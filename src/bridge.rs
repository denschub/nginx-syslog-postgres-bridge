use anyhow::{Error, Result};
use sqlx::{PgPool, postgres::PgQueryResult};
use tokio::{
    net::UdpSocket,
    sync::mpsc::{Receiver, Sender, channel},
    time::Duration,
};
use tracing::{debug, error, info, trace, warn};

use crate::{AccessLogColumnVecs, parsers::AccessLogEntry, settings::Settings};

pub struct Bridge {}

impl Bridge {
    pub async fn run(db_pool: PgPool, settings: Settings, udp_socket: UdpSocket) -> Result<()> {
        let (tx, rx) = channel::<String>(settings.queue_size);

        let udp_receiver = UdpReceiver::new(tx, udp_socket);
        let receiving_loop = tokio::spawn(async move { udp_receiver.run().await });

        let mut queue_item_storer = QueueItemStorer::new(
            db_pool,
            settings.insert_batch_size,
            settings.insert_timeout,
            rx,
        );
        let storing_loop = tokio::spawn(async move { queue_item_storer.run().await });

        tokio::select! {
            _ = receiving_loop => {},
            _ = storing_loop => {},
        };

        Ok(())
    }
}

pub struct UdpReceiver {
    received_sender: Sender<String>,
    socket: UdpSocket,
}

impl UdpReceiver {
    pub fn new(received_sender: Sender<String>, socket: UdpSocket) -> Self {
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
                debug!("Received {} bytes from {}", len, addr);

                let buf = buf[0..len].to_owned();
                let tx_clone = self.received_sender.clone();
                tokio::spawn(async move {
                    if let Ok(line) = String::from_utf8(buf) {
                        trace!("Raw message: `{}`", line);
                        // Silently drop send errors. This will fail if
                        // There's too much traffic, but if that's the case,
                        // spamming things to STDOUT doesn't help.
                        let _ = tx_clone.try_send(line);
                    }
                });
            }
        }
    }
}

struct QueueItemStorer {
    db_pool: PgPool,
    insert_batch_size: usize,
    insert_timeout: Duration,
    insert_field_vecs: AccessLogColumnVecs,
    receiver: Receiver<String>,
}

impl QueueItemStorer {
    pub fn new(
        db_pool: PgPool,
        insert_batch_size: usize,
        insert_timeout: u64,
        receiver: Receiver<String>,
    ) -> Self {
        Self {
            db_pool,
            insert_batch_size,
            insert_timeout: Duration::from_millis(insert_timeout),
            insert_field_vecs: AccessLogColumnVecs::with_capacity(insert_batch_size),
            receiver,
        }
    }

    async fn store_batch(&mut self, batch: &Vec<String>) -> Result<PgQueryResult, sqlx::Error> {
        self.insert_field_vecs.clear();
        for line in batch {
            if let Ok(entry) = Self::parse_datagram(line) {
                self.insert_field_vecs.push(entry);
            }
        }

        // Note: If any columns are added, removed, renamed, reorderd, or
        // otherwise touched, make sure to update [AccessLogColumnVecs].
        let query = sqlx::query(
            r#"
            INSERT INTO access_log (
                id, hostname, event_ts, server_name, server_port, client_addr, client_forwarded_for, client_referer,
                client_ua, req_host, req_length, req_method, req_proto, req_scheme, req_uri, res_body_length,
                res_duration, res_length, res_status, upstream_addr, upstream_bytes_received, upstream_bytes_sent,
                upstream_cache_status, upstream_connect_time, upstream_host, upstream_response_length,
                upstream_response_time, upstream_status
            ) SELECT * FROM UNNEST(
                $1::uuid[], $2::text[], $3::timestamptz[], $4::text[], $5::int4[], $6::text[], $7::text[], $8::text[],
                $9::text[], $10::text[], $11::int8[], $12::text[], $13::text[], $14::text[], $15::text[], $16::int8[],
                $17::float8[], $18::int8[], $19::int4[], $20::text[], $21::int8[], $22::int8[], $23::text[],
                $24::float8[], $25::text[], $26::int8[], $27::float8[], $28::int4[]
            )"#,
        );

        self.insert_field_vecs
            .bind_all(query)
            .execute(&self.db_pool)
            .await
    }

    pub async fn run(&mut self) {
        let mut batch: Vec<String> = Vec::with_capacity(self.insert_batch_size);
        loop {
            let received = self
                .receiver
                .recv_many(&mut batch, self.insert_batch_size)
                .await;
            if received < 1 {
                warn!("Channel closed, exiting storer loop...");
                return;
            }

            let mut batch_size = batch.len();
            if batch_size < self.insert_batch_size {
                debug!("Insert batch not yet full, waiting for more or timeout...");
                let _ = tokio::time::timeout(self.insert_timeout, async {
                    while batch_size < self.insert_batch_size {
                        let remaining = self.insert_batch_size - batch_size;
                        let _ = self.receiver.recv_many(&mut batch, remaining).await;
                        batch_size = batch.len();
                    }
                })
                .await;
            }

            if let Err(err) = self.store_batch(&batch).await {
                error!("Inserting into database failed: {:?}", err);
            }

            info!("Processed batch of {} entries", batch_size);
            batch.clear();
        }
    }

    fn parse_datagram(datagram: &str) -> Result<AccessLogEntry> {
        // at the moment, I'm completely ignoring everything provided by syslog
        // except the message. I could skip the syslog parsing, and just look for
        // the opening {, then read from there.
        // However, in the future, I might expand this with the ability to handle
        // error_log as well... so let's keep this for now.
        let syslog = syslog_loose::parse_message(datagram, syslog_loose::Variant::Either);
        serde_json::from_str(syslog.msg).map_err(Error::msg)
    }
}
