//! Knoa: an intelligent, AI-organized bookmark library for the desktop.

mod app;
mod paths;
mod services;
mod state;
mod ui;

fn main() {
    init_tracing();
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "starting Knoa");

    gpui_platform::application()
        .with_assets(gpui_kit_assets::AllAssets)
        .run(|cx| {
            if let Err(err) = app::init(cx) {
                tracing::error!("failed to start: {err:#}");
                cx.quit();
            }
        });
}

/// Logs go to stderr; filter with `RUST_LOG` (defaults to `knoa=info`).
fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("knoa=info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
