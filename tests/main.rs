use sqlx::PgPool;

mod helpers;
use crate::helpers::*;

const VALID_DATAGRAM_STATIC: &str = r#"<190>Aug 16 18:35:53 nginx: {"hostname":"a970744801bb","ts":"1660674953.230","server":{"name":"_","port":"80"},"client":{"addr":"172.19.0.1","forwarded_for":"","referer":"","ua":"Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:105.0) Gecko/20100101 Firefox/105.0"},"req":{"host":"localhost","length":"1703","method":"GET","proto":"HTTP/1.1","scheme":"http","uri":"/static_file_example"},"res":{"body_length":"0","duration":"0.000","length":"180","status":"304"},"upstream":{"addr":"","bytes_received":"","bytes_sent":"","cache_status":"","connect_time":"","host":"","response_length":"","response_time":"","status":""}}"#;
const VALID_DATAGRAM_UPSTREAM: &str = r#"<190>Aug 16 18:36:32 nginx: {"hostname":"a970744801bb","ts":"1660674992.468","server":{"name":"_","port":"80"},"client":{"addr":"172.19.0.1","forwarded_for":"","referer":"","ua":"Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:105.0) Gecko/20100101 Firefox/105.0"},"req":{"host":"localhost","length":"1658","method":"GET","proto":"HTTP/1.1","scheme":"http","uri":"/upstream_proxy_example"},"res":{"body_length":"648","duration":"0.254","length":"1044","status":"200"},"upstream":{"addr":"93.184.216.34:80","bytes_received":"1041","bytes_sent":"1705","cache_status":"","connect_time":"0.128","host":"example.com","response_length":"648","response_time":"0.253","status":"200"}}"#;

#[sqlx::test]
async fn stores_valid_datagram_for_static_requests(db_pool: PgPool) {
    let server_addr = spawn_test_server(db_pool.clone()).await;

    send_datagram(VALID_DATAGRAM_STATIC.as_bytes(), server_addr).await;

    wait_for_insert().await;
    let _ = sqlx::query("SELECT * FROM access_log")
        .fetch_one(&db_pool)
        .await
        .expect("did not find stored access_log database row");
}

#[sqlx::test]
async fn stores_valid_datagram_with_upstream_data(db_pool: PgPool) {
    let server_addr = spawn_test_server(db_pool.clone()).await;

    send_datagram(VALID_DATAGRAM_UPSTREAM.as_bytes(), server_addr).await;

    wait_for_insert().await;
    let _ = sqlx::query("SELECT * FROM access_log")
        .fetch_optional(&db_pool)
        .await
        .expect("did not find stored access_log database row");
}
