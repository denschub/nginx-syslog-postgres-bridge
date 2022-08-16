use anyhow::{Error, Result};
use log::{info, trace};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::{
    net::UdpSocket,
    sync::mpsc::{Receiver, Sender},
};

use crate::{parsers::AccessLogEntry, Settings};

pub struct Bridge {
    settings: Settings,
}

impl Bridge {
    pub async fn build(settings: Settings) -> Result<Self> {
        Ok(Self { settings })
    }

    pub async fn run(&self) -> Result<()> {
        let (tx, rx) = tokio::sync::mpsc::channel::<AccessLogEntry>(self.settings.queue_size);

        let udp_socket = tokio::net::UdpSocket::bind(&self.settings.listen_addr).await?;
        let db_pool = PgPoolOptions::new()
            .connect(&self.settings.database_uri)
            .await?;

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
        let syslog = syslog_loose::parse_message(datagram);
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

    pub async fn run(&mut self) {
        loop {
            if let Some(entry) = self.receiver.recv().await {
                let _ = entry.write_to_db(&self.db_pool).await;
            }
        }
    }
}
