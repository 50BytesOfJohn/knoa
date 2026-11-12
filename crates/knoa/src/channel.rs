//! Build identity, independent of Cargo's optimization profile.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Dev,
    Release,
}

impl Channel {
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

    pub fn app_id(self) -> &'static str {
        match self {
            Self::Dev => "dev.knoa.Knoa-Dev",
            Self::Release => "dev.knoa.Knoa",
        }
    }
}
