//! The native messaging host behind Knoa's browser extension.
//!
//! The browser starts it for each message or connection, so it works whether Knoa is
//! running or not: it writes to the library directly, then pokes the app so a running
//! Knoa shows the link and starts organizing it. The app registers it with every browser.

mod frame;
mod request;

use std::io;

fn main() -> anyhow::Result<()> {
    // Browsers pass the caller's origin as an argument. The host manifests already limit
    // callers to Knoa's extension, so it isn't needed.
    let (mut input, mut output) = (io::stdin().lock(), io::stdout().lock());
    while let Some(message) = frame::read(&mut input)? {
        frame::write(&mut output, &request::handle(&message))?;
    }
    Ok(())
}
