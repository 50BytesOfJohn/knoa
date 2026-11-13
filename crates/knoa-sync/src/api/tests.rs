//! Exercise the settings endpoints over HTTP, including session refresh and server errors.

use std::time::{Duration, Instant};

use futures::channel::mpsc;
use serde_json::{Value, json};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
};

use super::*;
use crate::{Config, Tokens};

#[tokio::test]
async fn settings_requests_retry_with_a_fresh_token_and_keep_the_put_body() {
    let fixture: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../sdk/fixtures/mcp-settings.json"
    )))
    .unwrap();
    let settings = fixture["settings"]["overrides"].clone();
    let cleared = fixture["settings"]["off"].clone();
    let mut answer = settings.clone();
    answer["url"] = fixture["url"].clone();
    let mut cleared_answer = cleared.clone();
    cleared_answer["url"] = fixture["url"].clone();
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let base = Url::parse(&format!("http://{}/", listener.local_addr().unwrap())).unwrap();
    let api = test_api(base);
    let expected_answer = answer.clone();
    let expected_cleared = cleared_answer.clone();
    let server = tokio::spawn(async move {
        let steps = [
            (
                "GET",
                "/v1/mcp/settings",
                "old-token",
                None,
                200,
                answer.clone(),
            ),
            (
                "PUT",
                "/v1/mcp/settings",
                "old-token",
                Some(settings.clone()),
                401,
                json!({"error":"unauthorized", "message":"Expired"}),
            ),
            (
                "POST",
                "/oauth/token",
                "",
                None,
                200,
                json!({"access_token":"new-token", "expires_in":3600}),
            ),
            (
                "PUT",
                "/v1/mcp/settings",
                "new-token",
                Some(settings),
                200,
                answer,
            ),
            (
                "PUT",
                "/v1/mcp/settings",
                "new-token",
                Some(cleared),
                200,
                cleared_answer,
            ),
            (
                "GET",
                "/v1/mcp/settings",
                "new-token",
                None,
                403,
                json!({"error":"forbidden", "message":"Settings unavailable"}),
            ),
        ];
        serve(listener, steps).await;
    });
    assert_eq!(
        serde_json::to_value(api.mcp_settings().await.unwrap()).unwrap(),
        expected_answer
    );
    let settings: McpSettings =
        serde_json::from_value(fixture["settings"]["overrides"].clone()).unwrap();
    assert_eq!(
        serde_json::to_value(api.set_mcp_settings(&settings).await.unwrap()).unwrap(),
        expected_answer
    );
    let cleared: McpSettings = serde_json::from_value(fixture["settings"]["off"].clone()).unwrap();
    assert_eq!(
        serde_json::to_value(api.set_mcp_settings(&cleared).await.unwrap()).unwrap(),
        expected_cleared
    );
    assert!(
        matches!(api.mcp_settings().await, Err(Error::Server { status: 403, code, message })
        if code == "forbidden" && message == "Settings unavailable")
    );
    server.await.unwrap();
}

type Exchange = (
    &'static str,
    &'static str,
    &'static str,
    Option<Value>,
    u16,
    Value,
);

async fn serve(listener: TcpListener, steps: [Exchange; 6]) {
    for (method, path, token, body, status, response) in steps {
        let (stream, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        assert_eq!(line.trim(), format!("{method} {path} HTTP/1.1"));
        let mut headers = String::new();
        loop {
            line.clear();
            reader.read_line(&mut line).await.unwrap();
            if line == "\r\n" {
                break;
            }
            headers.push_str(&line.to_ascii_lowercase());
        }
        if !token.is_empty() {
            assert!(headers.contains(&format!("authorization: bearer {token}\r\n")));
        }
        let length = headers
            .lines()
            .find_map(|line| line.strip_prefix("content-length: "))
            .map_or(0, |length| length.parse::<usize>().unwrap());
        let mut bytes = vec![0; length];
        reader.read_exact(&mut bytes).await.unwrap();
        if let Some(body) = body {
            assert!(headers.contains("content-type: application/json\r\n"));
            assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), body);
        } else if method == "POST" {
            let form: Vec<_> = url::form_urlencoded::parse(&bytes).into_owned().collect();
            assert!(form.contains(&("grant_type".into(), "refresh_token".into())));
            assert!(form.contains(&("refresh_token".into(), "refresh-token".into())));
        }
        let response = response.to_string();
        let wire = format!(
            "HTTP/1.1 {status} Test\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{response}",
            response.len()
        );
        reader.get_mut().write_all(wire.as_bytes()).await.unwrap();
    }
}

fn test_api(base: Url) -> Api {
    let (events, _) = mpsc::unbounded();
    Api::new(Arc::new(Session::new(
        reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap(),
        Config {
            api: base.clone(),
            issuer: base,
            client_id: "test-client".into(),
        },
        "refresh-token".into(),
        Some(Tokens {
            access: "old-token".into(),
            refresh: None,
            expires_at: Instant::now() + Duration::from_hours(1),
        }),
        events,
    )))
}
