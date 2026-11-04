# Knoa

An intelligent bookmark library for the desktop.

Bookmark in your browser as usual. Knoa imports those bookmarks, fetches each page and uses
an LLM to record what it is (an article, a video, a movie, ...), its key facts, tags and a
summary, and to file it into your folders following instructions you write in plain
language. You search and filter the library in a native macOS app.

> **Status: foundation.** The app opens a window with its title, and nothing else yet.
> Read [docs/ROADMAP.md](docs/ROADMAP.md) for what comes next.

## Docs

| Doc | What's in it |
| --- | --- |
| [docs/VISION.md](docs/VISION.md) | The product: what Knoa is, its principles, the library model |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | The technical design: crates, data flow, storage, sync, AI, search |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Milestones toward the public alpha |
| [docs/design/](docs/design/) | Approved screen designs |
| [docs/research/](docs/research/) | Product research behind design decisions |
| [docs/adr/](docs/adr/) | Architecture decision records: why each big choice was made |

## Tech

- **Rust** (edition 2024), pinned in [`rust-toolchain.toml`](rust-toolchain.toml)
- **[GPUI](https://gpui.rs)**, Zed's GPU-accelerated UI framework, consumed via the
  `gpui-pre` crates.io snapshots ([ADR 0001](docs/adr/0001-gpui-via-gpui-pre-snapshots.md))
- Planned: SQLite + FTS5 for storage and search, OpenRouter for LLM calls, gpui-component
  for UI primitives
  ([ARCHITECTURE](docs/ARCHITECTURE.md))

## Getting started

Requirements: macOS, [rustup](https://rustup.rs) (it installs the pinned toolchain itself),
and Xcode Command Line Tools (`xcode-select --install`). Optional:
[`taplo`](https://taplo.tamasfe.dev) for TOML formatting (`brew install taplo`).

```sh
make dev     # debug build, verbose logs (RUST_LOG=knoa=debug)
make run     # optimized build
```

The first build compiles GPUI and takes a few minutes; later builds are incremental.

## Development

```sh
make fmt     # format Rust + TOML
make lint    # clippy, pedantic, warnings are errors
make test    # tests
make check   # everything CI runs: fmt-check + lint + test
```

`cargo lint` and `cargo fmt-check` are also available as cargo aliases
([`.cargo/config.toml`](.cargo/config.toml)). CI runs the same checks on every push and pull
request ([`.github/workflows/ci.yml`](.github/workflows/ci.yml)).

### Project layout

```
crates/knoa/        the desktop app (binary)
  src/main.rs       entry point: logging, GPUI application
  src/app.rs        actions, key bindings, menus, main window
  src/ui/           views (one module per top-level view)
docs/               vision, architecture, roadmap, ADRs
```

The workspace is set up for more crates (`crates/*`); the planned crates are in
[ARCHITECTURE](docs/ARCHITECTURE.md#crates). Lints are defined once in the root
`Cargo.toml` under `[workspace.lints]` and inherited by every crate.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md); commits are signed off
under the DCO (`git commit -s`).

## License

Knoa is free software: you can redistribute it and/or modify it under the terms of the
[GNU General Public License v3.0 or later](LICENSE). Anyone who distributes Knoa, modified or
not, free or paid, must make the full source available under the same license.

The name "Knoa" and its logo are not covered by the license: forks must use their own name
and branding. See [TRADEMARK.md](TRADEMARK.md).
