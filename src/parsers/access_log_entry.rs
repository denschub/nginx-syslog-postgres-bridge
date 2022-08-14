use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::deserializers::*;

#[derive(Debug, Deserialize)]
pub struct AccessLogEntry {
    pub hostname: String,

    #[serde(deserialize_with = "datetime_from_mstimestamp")]
    pub ts: DateTime<Utc>,

    pub server: Server,

    pub client: Client,

    pub req: Req,

    pub res: Res,

    pub upstream: Upstream,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    #[serde(deserialize_with = "optional_normalized_string")]
    pub name: Option<String>,

    #[serde(deserialize_with = "optional_number_from_string")]
    pub port: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct Client {
    #[serde(deserialize_with = "optional_normalized_ip")]
    pub addr: Option<String>,

    #[serde(deserialize_with = "optional_normalized_string")]
    pub forwarded_for: Option<String>,

    #[serde(deserialize_with = "optional_normalized_string")]
    pub referer: Option<String>,

    #[serde(deserialize_with = "optional_normalized_string")]
    pub ua: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Req {
    #[serde(deserialize_with = "optional_normalized_string")]
    pub host: Option<String>,

    #[serde(deserialize_with = "optional_number_from_string")]
    pub length: Option<i64>,

    #[serde(deserialize_with = "optional_normalized_string")]
    pub method: Option<String>,

    #[serde(deserialize_with = "optional_normalized_string")]
    pub proto: Option<String>,

    #[serde(deserialize_with = "optional_normalized_string")]
    pub scheme: Option<String>,

    #[serde(deserialize_with = "optional_normalized_string")]
    pub uri: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Res {
    #[serde(deserialize_with = "optional_number_from_string")]
    pub body_length: Option<i64>,

    #[serde(deserialize_with = "optional_number_from_string")]
    pub duration: Option<f64>,

    #[serde(deserialize_with = "optional_number_from_string")]
    pub length: Option<i64>,

    #[serde(deserialize_with = "optional_number_from_string")]
    pub status: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct Upstream {
    #[serde(deserialize_with = "optional_normalized_ip")]
    pub addr: Option<String>,

    #[serde(deserialize_with = "optional_number_from_string")]
    pub bytes_received: Option<i64>,

    #[serde(deserialize_with = "optional_number_from_string")]
    pub bytes_sent: Option<i64>,

    #[serde(deserialize_with = "optional_normalized_string")]
    pub cache_status: Option<String>,

    #[serde(deserialize_with = "optional_number_from_string")]
    pub connect_time: Option<f64>,

    #[serde(deserialize_with = "optional_normalized_string")]
    pub host: Option<String>,

    #[serde(deserialize_with = "optional_number_from_string")]
    pub response_length: Option<i64>,

    #[serde(deserialize_with = "optional_number_from_string")]
    pub response_time: Option<f64>,

    #[serde(deserialize_with = "optional_number_from_string")]
    pub status: Option<i32>,
}

impl AccessLogEntry {
    pub async fn write_to_db(&self, db_pool: &PgPool) -> Result<()> {
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
            Uuid::new_v4(),
            self.ts,
            self.hostname,
            self.server.name,
            self.server.port,
            self.client.addr,
            self.client.forwarded_for,
            self.client.referer,
            self.client.ua,
            self.req.host,
            self.req.length,
            self.req.method,
            self.req.proto,
            self.req.scheme,
            self.req.uri,
            self.res.body_length,
            self.res.duration,
            self.res.length,
            self.res.status,
            self.upstream.addr,
            self.upstream.bytes_received,
            self.upstream.bytes_sent,
            self.upstream.cache_status,
            self.upstream.connect_time,
            self.upstream.host,
            self.upstream.response_length,
            self.upstream.response_time,
            self.upstream.status
        )
        .execute(db_pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    // So, this test suite is rather lackluster.
    // But then again... I know exactly what the inbound JSON will look like
    // as I'm also the one writing the nginx config. If I good that up,
    // I'll get a lot of warnings in the logs, and no entries in the database...
    // Also, serde_json itself has good error handling, and will just error out
    // if something funny gets passed into it...

    use super::*;

    #[test]
    fn deserializes_minimal_json() {
        let json = r#"{"hostname":"1b2dd316acb5","ts":"1660345167.135","server":{"name":"_","port":"80"},"client":{"addr":"172.18.0.1","forwarded_for":"","referer":"http://localhost:8080/","ua":"Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:105.0) Gecko/20100101 Firefox/105.0"},"req":{"host":"localhost","length":"1616","method":"GET","proto":"HTTP/1.1","scheme":"http","uri":"/favicon.ico"},"res":{"body_length":"615","duration":"0.000","length":"853","status":"200"},"upstream":{"addr":"","bytes_received":"","bytes_sent":"","cache_status":"","connect_time":"","host":"","response_length":"","response_time":"","status":""}}"#;
        let deserialized = serde_json::from_str::<AccessLogEntry>(json);
        assert!(deserialized.is_ok());
    }

    #[test]
    fn deserializes_json_with_upstream() {
        let json = r#"{"hostname":"1b2dd316acb5","ts":"1660345337.896","server":{"name":"_","port":"80"},"client":{"addr":"172.18.0.1","forwarded_for":"","referer":"","ua":"Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:105.0) Gecko/20100101 Firefox/105.0"},"req":{"host":"localhost","length":"1644","method":"GET","proto":"HTTP/1.1","scheme":"http","uri":"/upstream_test"},"res":{"body_length":"367","duration":"0.124","length":"616","status":"200"},"upstream":{"addr":"159.69.231.132:443","bytes_received":"571","bytes_sent":"1708","cache_status":"","connect_time":"0.096","host":"overengineer.dev","response_length":"355","response_time":"0.125","status":"200"}}"#;
        let deserialized = serde_json::from_str::<AccessLogEntry>(json);
        assert!(deserialized.is_ok());
    }

    #[test]
    fn is_err_for_junk() {
        let json = r#"{"hello": "world"}"#;
        let deserialized = serde_json::from_str::<AccessLogEntry>(json);
        assert!(deserialized.is_err());
    }
}
