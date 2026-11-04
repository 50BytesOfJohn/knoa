# Agent guide for Knoa

Knoa is a native macOS bookmark app in Rust using GPUI (Zed's UI framework). Read
`docs/VISION.md` for product intent and `docs/ARCHITECTURE.md` before structural changes.

## Commands

- `make check` runs everything CI runs (fmt-check, clippy with `-D warnings`, tests). Run it
  before declaring work done.
- `make fmt` formats Rust and TOML. `make dev` runs the app with debug logs.

## Conventions

- GPUI comes from the `gpui-pre` snapshot crates, renamed to `gpui` / `gpui_platform` in the
  root `Cargo.toml`. Its API changes between snapshots: check the source in
  `~/.cargo/registry/src/*/gpui-pre-<version>/` or docs.rs/gpui-pre rather than relying on
  memory of older `gpui` versions (e.g. the app starts with `gpui_platform::application()`,
  not `Application::new()`).
- Add dependencies to `[workspace.dependencies]` and reference them with `.workspace = true`.
- Lints live in `[workspace.lints]`; do not silence a lint locally without a comment saying why.
- Errors: `anyhow` in the app crate; typed errors (`thiserror`) in future library crates.
- Logging: `tracing`, never `println!`.
- Keep the UI on the main thread; do I/O and LLM calls on GPUI's background executor.
- Record significant technical decisions as a new ADR in `docs/adr/`.
