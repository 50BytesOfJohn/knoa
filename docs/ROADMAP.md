# Roadmap

Small, high-quality steps. Each milestone should leave the app working and the docs current.

## M0: Foundation (done)

- [x] Cargo workspace, Rust 2024, pinned toolchain
- [x] GPUI app that opens one window titled "Knoa" (Quit via ⌘Q and the app menu)
- [x] rustfmt, taplo, clippy (pedantic, warnings as errors), editorconfig
- [x] CI: format check, clippy, tests on macOS
- [x] Vision, architecture, roadmap and ADR docs

## M1: App shell and design system

- [ ] Theme tokens (colors, type scale, spacing), following the system light/dark appearance
- [ ] Decide on gpui-component vs. hand-rolled components (ADR)
- [ ] Three-pane layout skeleton: sidebar, item list, detail pane, custom titlebar
- [ ] Command palette and core key bindings
- [ ] Persist window size/position

## M2: Local library

- [ ] `knoa-core` domain types
- [ ] `knoa-store`: SQLite, migrations, repository API, tests
- [ ] Render real bookmarks in the list from the database (virtualized list)
- [ ] Manual add/remove for development

## M3: Browser sync

- [ ] Chromium-family reader (Chrome, Arc, Brave, Helium, ...) with profile discovery
- [ ] File watching with debounce; idempotent import
- [ ] Firefox-family reader (Firefox, Zen)
- [ ] Safari reader with a Full Disk Access onboarding step
- [ ] Onboarding: pick the browsers/profiles to sync

## M4: Rich previews

- [ ] Page fetch + Open Graph / oEmbed metadata
- [ ] Content-kind detection (video, image, article, repo, other)
- [ ] Thumbnail fetching and on-disk cache
- [ ] Per-kind presentation in list/grid, appearance settings per kind

## M5: AI organization (OpenRouter)

- [ ] API key onboarding, stored in Keychain
- [ ] Organizing instructions: written during onboarding, editable in settings
- [ ] `knoa-ai`: OpenRouter client with structured outputs
- [ ] Durable background job queue with retries and cost-aware batching
- [ ] Categories, tags, summaries; "why it's here" and one-click undo
- [ ] User corrections always win over AI on re-runs

## M6: Search

- [ ] FTS5 index and instant search UI
- [ ] Filters: kind, tag, collection, source, date

## Later

- [ ] Archive state and views
- [ ] Relevance check: periodically re-evaluate whether items are still worth reading
- [ ] Automatic mark/archive of stale or irrelevant items, guided by the instructions
- [ ] Semantic search / RAG with `sqlite-vec` embeddings; "ask your library"
- [ ] Claude / Codex subscription providers (check terms first)
- [ ] Browser extension for direct capture with notes
- [ ] App bundle, signing, notarization, auto-update
- [ ] Linux and Windows builds
