//! What can go wrong talking to Clerk or the API, sorted by what the caller should do.

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("the server answered {status}: {message}")]
    Server {
        status: u16,
        /// The API's error code, like `subscription_required`.
        code: String,
        message: String,
    },
    /// The refresh token was refused: the user has to sign in again.
    #[error("signed out")]
    SignedOut,
    #[error("sign-in didn't finish: {0}")]
    SignIn(String),
    /// A URL or header that can't be built from the configuration.
    #[error("{0}")]
    Config(String),
    #[error("sign-in timed out")]
    TimedOut,
    #[error("socket: {0}")]
    Socket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

impl Error {
    /// Trouble that may go away by itself: no network, or the server failing.
    pub fn is_transient(&self) -> bool {
        match self {
            Error::Network(_) | Error::Socket(_) | Error::Io(_) => true,
            Error::Server { status, .. } => *status >= 500 || *status == 429,
            Error::SignedOut | Error::SignIn(_) | Error::Config(_) | Error::TimedOut => false,
        }
    }
}
