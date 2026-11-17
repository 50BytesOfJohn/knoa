//! The browser extension's side door: `knoa-bridge`, the native messaging host bundled
//! with the app. At launch the app registers it with every installed browser, then listens
//! for its nudge after it saves a link.
//!
//! The bridge writes to the library itself, so the app only needs a nudge to show the
//! link and start organizing it. The nudge is a datagram on a Unix socket next to the
//! database; any message means "the library changed".

mod hosts;

use std::{os::unix::net::UnixDatagram, path::Path};

use futures::{StreamExt as _, channel::mpsc};
use gpui::{App, Entity};

use crate::{services::organizer::Organizer, state::library::Library};

/// Writes the browsers' host manifests off the UI thread.
pub fn register(cx: &App) {
    cx.background_executor()
        .spawn(async {
            if let Err(err) = hosts::register() {
                tracing::warn!("can't register with the browsers: {err:#}");
            }
        })
        .detach();
}

pub fn listen(socket: &Path, library: Entity<Library>, cx: &mut App) {
    // A socket file left by an earlier run would make the bind fail.
    let _ = std::fs::remove_file(socket);
    let listener = match UnixDatagram::bind(socket) {
        Ok(listener) => listener,
        Err(err) => {
            tracing::warn!(?socket, "not listening for the browser extension: {err}");
            return;
        }
    };
    let (tx, mut rx) = mpsc::unbounded();
    // A blocking receive would tie up one of GPUI's executor threads for good.
    std::thread::spawn(move || {
        let mut buf = [0; 64];
        loop {
            match listener.recv(&mut buf) {
                Ok(_) => {}
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(err) => {
                    tracing::warn!("stopped listening for the browser extension: {err}");
                    break;
                }
            }
            if tx.unbounded_send(()).is_err() {
                break;
            }
        }
    });
    cx.spawn(async move |cx| {
        while rx.next().await.is_some() {
            tracing::debug!("the browser extension changed the library");
            cx.update(|cx| {
                library.update(cx, Library::refresh);
                Organizer::wake(cx);
            });
        }
    })
    .detach();
}
