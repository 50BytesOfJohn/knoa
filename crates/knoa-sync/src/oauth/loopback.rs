//! The one-shot listener the browser is redirected to: `http://127.0.0.1:<port>/callback`
//! on a port the system picks (RFC 8252, section 7.3).

use std::time::Duration;

use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    net::{TcpListener, TcpStream},
};
use url::Url;

use crate::{Error, Result};

/// A request head longer than this isn't the browser coming back from sign-in.
const MAX_REQUEST: usize = 16 * 1024;
/// The browser sends its request at once; a connection that stalls must not hold up the
/// real callback behind it.
const READ_TIMEOUT: Duration = Duration::from_secs(5);

const SIGNED_IN: &str = "You're signed in. You can close this tab and return to Knoa.";
const NOT_SIGNED_IN: &str = "Signing in didn't finish. Return to Knoa and try again.";

pub(super) struct Loopback {
    listener: TcpListener,
    port: u16,
}

impl Loopback {
    pub async fn bind() -> Result<Loopback> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
        let port = listener.local_addr()?.port();
        Ok(Loopback { listener, port })
    }

    pub fn redirect_uri(&self) -> String {
        format!("http://127.0.0.1:{}/callback", self.port)
    }

    /// Answers requests until the callback arrives, and returns its code. Anything else
    /// the browser asks for, like a favicon, gets a 404. Any local process can connect, so
    /// a malformed request or a callback without this sign-in's state is turned away
    /// without ending the sign-in.
    pub async fn wait(self, state: &str) -> Result<String> {
        loop {
            let (mut stream, _) = self.listener.accept().await?;
            let Ok(target) = tokio::time::timeout(READ_TIMEOUT, read_target(&mut stream)).await
            else {
                continue;
            };
            let Some(target) = target else {
                respond(&mut stream, "400 Bad Request", "Bad request.").await;
                continue;
            };
            match parse_callback(&target, state) {
                Callback::Other => respond(&mut stream, "404 Not Found", "Not found.").await,
                Callback::Foreign => {
                    tracing::warn!("ignored a sign-in callback with another state");
                    respond(&mut stream, "400 Bad Request", NOT_SIGNED_IN).await;
                }
                Callback::Code(code) => {
                    respond(&mut stream, "200 OK", SIGNED_IN).await;
                    return Ok(code);
                }
                Callback::Failed(why) => {
                    respond(&mut stream, "400 Bad Request", NOT_SIGNED_IN).await;
                    return Err(Error::SignIn(why));
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Callback {
    /// Not the redirect: some other path.
    Other,
    /// A redirect without this sign-in's state: not ours to trust, whatever else it says.
    Foreign,
    Code(String),
    Failed(String),
}

/// Reads `target` (`/callback?code=..&state=..`) against the `state` this sign-in sent.
fn parse_callback(target: &str, state: &str) -> Callback {
    let Ok(url) = Url::parse("http://127.0.0.1").and_then(|base| base.join(target)) else {
        return Callback::Other;
    };
    if url.path() != "/callback" {
        return Callback::Other;
    }
    let param = |name: &str| {
        url.query_pairs()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.into_owned())
    };
    if param("state").as_deref() != Some(state) {
        return Callback::Foreign;
    }
    if let Some(error) = param("error") {
        let description = param("error_description").unwrap_or_default();
        return Callback::Failed(format!("{error} {description}").trim().to_owned());
    }
    match param("code") {
        Some(code) if !code.is_empty() => Callback::Code(code),
        _ => Callback::Failed("the browser came back without a code".into()),
    }
}

/// The request line's target, once the whole head has arrived.
async fn read_target(stream: &mut TcpStream) -> Option<String> {
    let mut head = Vec::new();
    let mut buffer = [0u8; 2048];
    while !head.windows(4).any(|w| w == b"\r\n\r\n") {
        let read = stream.read(&mut buffer).await.ok()?;
        if read == 0 || head.len() + read > MAX_REQUEST {
            return None;
        }
        head.extend_from_slice(&buffer[..read]);
    }
    let head = String::from_utf8_lossy(&head);
    let mut line = head.lines().next()?.split_whitespace();
    match (line.next(), line.next()) {
        (Some("GET"), Some(target)) => Some(target.to_owned()),
        _ => None,
    }
}

async fn respond(stream: &mut TcpStream, status: &str, message: &str) {
    let body = format!(
        "<!doctype html><meta charset=utf-8><title>Knoa</title>\
         <body style=\"font:16px -apple-system,sans-serif;margin:20vh auto;max-width:28em;text-align:center\">\
         <p>{message}</p></body>"
    );
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    if let Err(err) = stream.write_all(response.as_bytes()).await {
        tracing::debug!("couldn't answer the browser: {err}");
    }
    let _ = stream.shutdown().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_callback_with_our_state_yields_its_code() {
        assert_eq!(
            parse_callback("/callback?code=abc&state=s1", "s1"),
            Callback::Code("abc".into())
        );
        assert_eq!(
            parse_callback("/callback?code=abc&state=other", "s1"),
            Callback::Foreign
        );
        assert!(matches!(
            parse_callback("/callback?error=access_denied&state=s1", "s1"),
            Callback::Failed(why) if why.contains("access_denied")
        ));
        assert_eq!(parse_callback("/favicon.ico", "s1"), Callback::Other);
    }

    #[tokio::test]
    async fn a_callback_from_another_sign_in_does_not_end_this_one() {
        let loopback = Loopback::bind().await.unwrap();
        let port = loopback.port;
        let waiting = tokio::spawn(async move { loopback.wait("s1").await });
        for target in [
            "/callback?code=evil&state=other",
            "/callback?code=abc&state=s1",
        ] {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
            let request = format!("GET {target} HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
            stream.write_all(request.as_bytes()).await.unwrap();
            let mut answer = String::new();
            stream.read_to_string(&mut answer).await.unwrap();
        }
        assert_eq!(waiting.await.unwrap().unwrap(), "abc");
    }
}
