use anyhow::{Error, Result};
use log::{info, trace, warn};
use sqlx::postgres::PgPoolOptions;

use crate::{parsers::AccessLogEntry, Settings};

pub struct Bridge {
    settings: Settings,
}

impl Bridge {
    pub async fn build(settings: Settings) -> Result<Self> {
        Ok(Self { settings })
    }

    pub async fn run(&self) -> Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<AccessLogEntry>(self.settings.queue_size);

        let udp_socket = tokio::net::UdpSocket::bind(&self.settings.listen_addr).await?;
        let db_pool = PgPoolOptions::new()
            .connect(&self.settings.database_uri)
            .await?;

        let receiving_loop = tokio::spawn(async move {
            // As per RFC5426, a syslog-via-udp message can only ever be one UDP
            // datagram long, not more. So we know the maximum ever length of that,
            // and the size is small enough to just allocate everything.
            let mut buf = [0; 65535];

            loop {
                let (len, addr) = udp_socket.recv_from(&mut buf).await.unwrap();
                info!("Received {} bytes from {}", len, addr);

                let buf = buf[0..len].to_owned();
                let tx_clone = tx.clone();
                tokio::spawn(async move {
                    if let Ok(line) = std::str::from_utf8(&buf) {
                        trace!("Raw message: `{}`", line);
                        match parse_datagram(line).await {
                            Ok(entry) => match tx_clone.try_send(entry) {
                                Ok(_) => {}
                                Err(_) => {
                                    warn!("Can't send message to the queue. Dropping.");
                                }
                            },
                            Err(e) => {
                                warn!("Failed to process message: {}", e);
                            }
                        }
                    }
                });
            }
        });

        let storing_loop = tokio::spawn(async move {
            loop {
                let maybe_entry = rx.recv().await;
                if let Some(entry) = maybe_entry {
                    let _ = entry.write_to_db(&db_pool).await;
                }
            }
        });

        tokio::select! {
            _ = receiving_loop => {},
            _ = storing_loop => {},
        };

        Ok(())
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
