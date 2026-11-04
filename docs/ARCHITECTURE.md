# Architecture

The design for the alpha. Only the app shell is built so far. When part of this becomes
code, update this file in the same change. Product decisions are in [VISION](VISION.md).

## Overview

```
 Chromium bookmarks        Knoa
 (Bookmarks JSON)    ┌────────────────────────────────────────────────────┐
 Chrome, Arc,   ───▶ │ sync ──▶ import ──▶ fetch ──▶ extract ──▶ LLM      │
 Brave, Helium       │ (watch)  (dedupe)                     (OpenRouter) │
                     │             │                              │       │
                     │             ▼                              │       │
                     │          SQLite ◀───────────────────────────┘       │
                     │             │                                      │
                     │             ▼                                      │
                     │          GPUI views                                │
                     └────────────────────────────────────────────────────┘
```

One process. The UI thread only renders and handles input:

- Database work runs on GPUI's background executor.
- Network work (page fetches, LLM calls) runs on a dedicated tokio runtime
  ([ADR 0006](adr/0006-tokio-runtime-for-network.md)).
- Results return to views through channels and entity updates. Nothing on the UI thread
  waits for disk or network.

## Crates

| Crate | Role | Depends on GPUI? |
| --- | --- | --- |
| `knoa` (exists) | App binary: windows, views, actions; sync and AI as modules | yes |
| `knoa-core` | Domain types (`Bookmark`, `Kind`, `Field`, `Folder`, `Query`), kind definitions | no |
| `knoa-store` | SQLite schema, migrations, queries, full-text search | no |

## Data model

See [ADR 0005](adr/0005-typed-bookmarks-and-folders.md).

- Kind definitions (kinds and their typed fields) are data in an embedded file in
  `knoa-core`.
- A bookmark stores its kind, `fields` and `extras`. Fields and extras are JSON columns;
  field filters use SQLite's `json_extract`.
- Tags and folders are many-to-many with bookmarks.
- Every AI-set value and folder link records its origin (AI or user). AI re-runs never
  overwrite user values.
- Folders have a name, an optional description and an `auto_assign` flag (default on).
- Folder exclusions store (bookmark, folder) pairs the user removed, so the AI never adds
  them back.

The exact schema is written with `knoa-store`.

## Storage

SQLite via `rusqlite` (`bundled`), WAL mode, embedded versioned migrations. The database is
`~/Library/Application Support/Knoa/knoa.db`. Thumbnails are cached as files in the app's
cache directory. See [ADR 0002](adr/0002-sqlite-local-first-storage.md).

## Sync

- Reads Chromium-family bookmark files
  (`~/Library/Application Support/<vendor>/<Profile>/Bookmarks`) for Chrome, Arc, Brave,
  Helium and others, with profile discovery.
- Watches them with `notify`, debounced.
- One-way, browser → Knoa. Bookmarks are keyed on URL plus the browser's GUID, so
  re-imports are idempotent. The browser folder path is stored and passed to the AI as a
  hint.
- A bookmark deleted in the browser is marked, not deleted.
- The first import stores metadata only. AI processing of those bookmarks starts when the
  user triggers it. Later bookmarks are processed automatically.

## Enrichment

Each bookmark goes through a durable job queue (retries with backoff, about 4 concurrent
network requests), so quitting the app loses no work.

1. **Fetch** the page with `reqwest` (plain HTTP, with timeout and size limit).
2. **Extract** metadata, in this order:
   - site-specific extractors for popular sites (YouTube, Vimeo, IMDb, GitHub, ...);
   - structured data on the page: schema.org JSON-LD, Open Graph, oEmbed;
   - LLM extraction from the page text when the above is not enough.
3. **Organize** with the LLM through OpenRouter Chat Completions with
   `response_format: { type: "json_schema", strict: true }`. Output: kind, fields, extras,
   tags, summary, folders. Input: the user's instructions, descriptions of auto-assign
   folders, existing tags, the extracted metadata and truncated page text.
4. **Store** the result with the model name and instruction version.

The default models are small and cheap; result quality matters more than cost. Before the
queue is built, a proof of concept compares one request per bookmark with two or three
focused requests (for example, folder matching on its own) on real bookmarks.

Folder "find bookmarks" and re-scan run folder matching over bookmarks that are already
processed. They only add.

The OpenRouter client is a small hand-written module
([ADR 0003](adr/0003-openrouter-thin-client.md)). The API key is stored in the macOS
Keychain (`keyring` crate).

## Search

- SQLite FTS5 over titles, URLs, summaries, tags, field values, extras and page text, with
  prefix matching and BM25 ranking.
- A `Query` in `knoa-core` combines text with filters (kind, site, folder, tag, field
  comparisons such as `runtime > 2h`). `knoa-store` turns it into SQL. The search bar edits
  this `Query`.

## UI

- Main screen: a floating sidebar (workspace, places, folders), a floating search pill
  with kind chips, masonry cards grouped by date with a date rail, and a floating detail
  panel. The approved layout is in the [main screen design](design/main-screen.md); the
  research behind it is in [research](research/main-screen.md).
- Masonry, glass panels over a wallpaper and the date rail are Knoa's own elements
  ([ADR 0007](adr/0007-main-screen-rendering.md)).
- Components: [gpui-component](https://github.com/longbridge/gpui-component) for input,
  virtualized lists, scrollbars and popovers, styled with Knoa's theme. Cards, search bar,
  detail view, command palette and animations are Knoa's own
  ([ADR 0004](adr/0004-hybrid-ui-components.md)).
- Keyboard-first: command palette and shortcuts for all main actions.

## Distribution

macOS only. The alpha is distributed as a signed and notarized app bundle.
