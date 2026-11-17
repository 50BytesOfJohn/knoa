//! Which Knoa this build is, and where it keeps its files. The app and `knoa-bridge` both
//! read the library through these, so the bridge always writes to the app's library.

mod channel;
mod paths;

pub use channel::Channel;
pub use paths::Paths;
