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
| [0005](0005-typed-bookmarks-and-folders.md) | Typed bookmarks and one-level folders | Folders superseded by 0017 |
| [0006](0006-tokio-runtime-for-network.md) | A dedicated tokio runtime for network work | Accepted |
| [0007](0007-main-screen-rendering.md) | Masonry, glass panels and the date rail in GPUI | Accepted |
| [0008](0008-credential-storage.md) | Keychain in signed builds, plain file in dev builds | Accepted |
| [0009](0009-provider-neutral-ai.md) | Provider-neutral AI behind two traits | Accepted |
| [0010](0010-bookmark-sync-by-file-watch.md) | Bookmark sync by file watch, with a slow rescan | Accepted |
| [0011](0011-workspaces-fed-by-browsers.md) | Workspaces are fed by browsers, with an optional guideline | Accepted |
| [0012](0012-deleting-bookmarks-through-trash.md) | Deleting bookmarks goes through Trash; no archive | Accepted |
| [0013](0013-kind-before-fields.md) | Pick the kind first, then extract its fields | Accepted |
| [0014](0014-releases-and-updates.md) | Releases on GitHub, updates through Sparkle | Accepted |
| [0015](0015-separate-read-connection.md) | One writer and one read-only reader for the database | Accepted |
| [0016](0016-images-seen-from-derived-copies.md) | Images are stored and shown to the AI as derived copies | Accepted |
| [0017](0017-nested-folders-and-emoji-icons.md) | Folders nest three levels deep, and an icon can be an emoji | Accepted |
| [0018](0018-native-phone-apps.md) | Native phone apps, each with its own sync client | Accepted |
| [0019](0019-browser-extension.md) | A browser extension with a local and a cloud mode | Accepted |
