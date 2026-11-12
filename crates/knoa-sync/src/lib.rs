//! Knoa's cloud client: signing in with Clerk through the system browser, keeping the
//! access token fresh, and the Knoa API's endpoints and socket.
//!
//! Plain async code on tokio, with no GPUI dependency, like `knoa-fetch` and `knoa-ai`.
//! The app decides when to sync; this crate only knows how to talk to the server.

mod api;
mod error;
mod oauth;
mod session;
mod socket;

pub use api::Api;
pub use error::{Error, Result};
pub use oauth::{SignIn, Tokens, revoke, user_email};
pub use session::{Session, SessionEvent};
pub use socket::{SocketEvent, run_socket};
use url::Url;

/// Where the account lives: the Knoa API and the Clerk instance that signs users in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub api: Url,
    /// Clerk's Frontend API, which is also its OAuth issuer.
    pub issuer: Url,
    pub client_id: String,
}

/// One HTTP client for every request to the API and to Clerk.
pub fn http_client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .user_agent(concat!("Knoa/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(60))
        .build()?)
}
