//! Build identity, independent of Cargo's optimization profile.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Dev,
    Release,
}

impl Channel {
    /// Set at build time by `KNOA_CHANNEL=release`; every other build is Knoa Dev.
    pub fn current() -> Self {
        match option_env!("KNOA_CHANNEL") {
            Some("release") => Self::Release,
            _ => Self::Dev,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Dev => "Knoa Dev",
            Self::Release => "Knoa",
        }
    }

    /// The native messaging host the browser extension talks to.
    pub fn bridge_host(self) -> &'static str {
        match self {
            Self::Dev => "dev.knoa.bridge_dev",
            Self::Release => "dev.knoa.bridge",
        }
    }

    pub fn app_id(self) -> &'static str {
        match self {
            Self::Dev => "dev.knoa.Knoa-Dev",
            Self::Release => "dev.knoa.Knoa",
        }
    }
}
