//! Long-running work behind the UI: Cloud mode, browser sync, the organizing queue, AI connection,
//! credentials, the network runtime, opening links, cleaning up at launch, refreshing
//! thumbnails, software updates and AI usage.

pub mod ai;
pub mod cloud;
pub mod credentials;
mod generation;
pub mod housekeeping;
pub mod links;
pub mod network;
pub mod organizer;
pub mod sync;
pub mod thumbnails;
pub mod updates;
pub mod usage;
