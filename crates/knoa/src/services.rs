//! Long-running work behind the UI: browser sync, the organizing queue, AI connection,
//! credentials, the network runtime, opening links, cleaning up at launch and software
//! updates.

pub mod ai;
pub mod credentials;
pub mod housekeeping;
pub mod links;
pub mod network;
pub mod organizer;
pub mod sync;
pub mod updates;
