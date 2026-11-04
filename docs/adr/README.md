# Architecture decision records

One short file per significant decision: context, decision, consequences. Numbered in
order. Once a decision is implemented, change it by adding a new ADR that supersedes the
old one.

| # | Decision | Status |
| --- | --- | --- |
| [0001](0001-gpui-via-gpui-pre-snapshots.md) | GPUI via the `gpui-pre` snapshot crates | Accepted |
| [0002](0002-sqlite-local-first-storage.md) | SQLite for local storage, FTS5 for search | Accepted |
| [0003](0003-openrouter-thin-client.md) | OpenRouter through a thin hand-written client | Accepted |
| [0004](0004-hybrid-ui-components.md) | gpui-component for primitives, own design on top | Accepted |
| [0005](0005-typed-bookmarks-and-folders.md) | Typed bookmarks and one-level folders | Accepted |
| [0006](0006-tokio-runtime-for-network.md) | A dedicated tokio runtime for network work | Accepted |
| [0007](0007-main-screen-rendering.md) | Masonry, glass panels and the date rail in GPUI | Accepted |
| [0008](0008-credential-storage.md) | Keychain in signed builds, plain file in dev builds | Accepted |
| [0009](0009-provider-neutral-ai.md) | Provider-neutral AI behind two traits | Accepted |
| [0010](0010-bookmark-sync-by-file-watch.md) | Bookmark sync by file watch, with a slow rescan | Accepted |
