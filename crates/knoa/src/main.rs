//! Knoa: an intelligent, AI-organized bookmark library for the desktop.

mod app;
mod assets;
mod menus;
mod paths;
mod services;
mod startup_error;
mod state;
mod ui;

fn main() {
    init_tracing();
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "starting Knoa");

    gpui_platform::application()
        .with_assets(assets::Assets)
        .run(|cx| {
            if let Err(err) = app::init(cx) {
                startup_error::report(&err, cx);
            }
        });
}

/// Logs go to stderr; filter with `RUST_LOG` (defaults to `knoa=info`).
fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("knoa=info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
