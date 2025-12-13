use chrono::{DateTime, Utc};
use serde::Deserialize;

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
