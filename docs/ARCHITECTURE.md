# Architecture

This is the **planned** design. Only the app shell exists today; sections below say what is
built and what is intended. When a plan becomes code, update this file in the same change.

## Overview

```
 Browser bookmark stores                    Knoa
 ───────────────────────     ┌──────────────────────────────────────────────────┐
 Chrome / Arc / Brave /      │  sync        ingest         enrich (AI)          │
 Helium (Bookmarks JSON)  ──▶│  watcher ──▶ normalize ──▶  fetch page ──▶ LLM ──┤
 Firefox / Zen               │              dedupe         metadata     (OpenRouter)
 (places.sqlite)             │                 │                             │  │
 Safari (Bookmarks.plist)    │                 ▼                             ▼  │
                             │              SQLite (bookmarks, tags, FTS5, jobs)│
                             │                 │                                │
                             │                 ▼                                │
                             │              GPUI views (library, detail, search)│
                             └──────────────────────────────────────────────────┘
```

Everything runs locally in one process. The UI thread only renders and reacts; sync, fetching,
LLM calls and database writes run on GPUI's background executor, and results come back to
views as entity updates.

## Crates

The workspace is `crates/*`. Today there is one crate; the intended split, introduced only
when there's real code for each, is:

| Crate | Role | Depends on GPUI? |
| --- | --- | --- |
| `knoa` (exists) | The app binary: windows, views, actions, wiring | yes |
| `knoa-core` | Domain types (`Bookmark`, `Tag`, `Collection`, `ContentKind`), pure logic | no |
| `knoa-store` | SQLite schema, migrations, queries, FTS | no |
| `knoa-sync` | Readers + watchers for each browser's bookmark store | no |
| `knoa-ai` | OpenRouter client, prompts, structured-output schemas | no |

Keeping everything except the app crate free of GPUI means the hard logic is testable with
plain `cargo test`, and a CLI or other frontend stays possible.

## Bookmark sync

Goal: the user keeps clicking the browser's bookmark icon and Knoa sees the result. No
browser extension is needed for v1; Knoa reads each browser's own bookmark store and watches
it for changes (`notify` crate, debounced).

| Browser family | Store | Notes |
| --- | --- | --- |
| Chromium (Chrome, Arc, Brave, Edge, Helium, ...) | `~/Library/Application Support/<vendor>/<Profile>/Bookmarks` (JSON) | Easy: plain JSON, rewritten on every change. Multiple profiles. |
| Firefox family (Firefox, Zen) | `places.sqlite` in the profile dir | Locked while the browser runs: open read-only with `immutable=1`, or copy first. |
| Safari | `~/Library/Safari/Bookmarks.plist` | Needs Full Disk Access; the onboarding must explain and request it. |

Each source (browser + profile) maps to a workspace (see [Workspaces](#workspaces)); unmapped
sources go to the default workspace, or to whichever workspace the AI routes them to.

Sync is **one-way** at first (browser → Knoa), keyed on URL plus the browser's own GUID, so
re-imports are idempotent. A browser extension that pushes bookmarks in directly (and adds
"save to Knoa" with a note) is a later option, not a requirement.

## Storage

**SQLite** via `rusqlite` with the `bundled` feature (no system dependency), WAL mode, and
embedded, versioned migrations (`rusqlite_migration` or `refinery`). The file lives in
`~/Library/Application Support/Knoa/knoa.db` (resolved with the `directories` crate).

Sketch of the schema:

- `workspaces`: id, name, icon, position, `hidden`, `locked` (Touch ID), organizing
  instructions
- `sources`: synced browser profiles, each mapped to a workspace
- `bookmarks`: id, workspace_id, url, canonical_url, title, source_browser, source_guid, added_at,
  content_kind, status (inbox / library / archived), metadata JSON (Open Graph, duration,
  dimensions), summary, reading_time, enriched_at, enrichment_model
- `tags`, `bookmark_tags`: many-to-many, with `origin` (ai / user) so user edits win
- `collections`: AI- or user-created groupings
- `jobs`: durable queue for enrichment work (retries, backoff), so nothing is lost when the
  app quits mid-run
- `settings`: global preferences and global organizing instructions (per-workspace ones
  live on `workspaces`; both versioned, so we can see which instructions produced which
  decisions)
- `bookmarks_fts`: FTS5 virtual table over title, url, summary, tags, extracted text

Thumbnails and preview images are cached on disk under the app's cache directory, not in the
database.

Why SQLite: one file, transactional, very fast for this data size, first-class full-text
search, and a proven path to vector search later. See
[ADR 0002](adr/0002-sqlite-local-first-storage.md).

## Workspaces

A workspace is a column on the rows it owns (`bookmarks`, `collections`, `tags`), not a
separate database file. One file keeps moving items, global search and backups simple, and
every query is scoped by `workspace_id`. A default workspace always exists.

- **Routing:** a new bookmark goes to its source's workspace. Bookmarks from an unmapped
  source can be routed by the LLM, which sees the list of workspaces and their
  instructions.
- **Instructions:** the prompt combines global instructions with the target workspace's
  instructions.
- **Hidden workspaces** are excluded from the switcher, cross-workspace search, notifications
  and previews until revealed for the session. A locked one also needs Touch ID
  (`LocalAuthentication`) to reveal. This is privacy from casual view, not encryption: the
  data is in the same database file. If real encryption is ever wanted, it means a separate
  encrypted database (SQLCipher) for that workspace, and an ADR.

## AI enrichment

Pipeline per bookmark:

1. **Fetch** the page (`reqwest`) with a timeout and size cap. Extract Open Graph / oEmbed
   metadata and main text (`readability`-style extraction). Detect the content kind from the
   URL (YouTube, image extensions, ...) and the metadata.
2. **Ask the LLM** for a structured result using OpenRouter's Chat Completions API with
   `response_format: { type: "json_schema", strict: true }`: category/collection, tags,
   one-line summary, content kind confirmation, language. The prompt contains the user's
   organizing instructions, the existing collections and tags (so the model reuses them
   instead of inventing near-duplicates), and the extracted page content (truncated).
3. **Store** the result with the model name and instruction version. User edits are never
   overwritten by re-enrichment.

The OpenRouter client is a small hand-written module on top of `reqwest` + `serde`: the API
is OpenAI-compatible and we need only a handful of endpoints, so a thin client is easier to
own than a third-party SDK. Requests send `HTTP-Referer` and `X-Title` headers for app
attribution. Model choice is a setting with a good, cheap default.

The API key is stored in the **macOS Keychain** (`keyring` crate), never in the database or
a config file.

**Async bridge.** GPUI has its own executor rather than tokio. Two workable options, to be
decided when the AI crate lands (ADR to follow):
- use GPUI's `http_client` abstraction with the `gpui-pre-reqwest-client` crate, as Zed does;
  or
- keep `knoa-ai` runtime-agnostic and run it on a small dedicated tokio runtime, handing
  results back to GPUI through channels.

## Search

- **v1: FTS5** keyword search over titles, URLs, summaries, tags and extracted text, with
  prefix matching and BM25 ranking. Instant, offline, no AI cost.
- **Later: semantic search / RAG.** Store an embedding per bookmark (title + summary + key
  text) in SQLite using the `sqlite-vec` extension. Embeddings come from OpenRouter's
  embeddings endpoint, or a local model (e.g. via `fastembed`) to keep search fully offline.
  Hybrid ranking (FTS + vector) for search, and retrieval of top-k bookmarks as context for
  "ask your library" questions.

## UI

GPUI views composed as entities. The planned main window is a classic three-pane layout:
sidebar (smart views, collections, tags), list or grid of items (presentation chosen by
content kind), and a detail pane with the preview, summary, tags and "why it's here".
Keyboard-first: command palette, quick search, everything reachable without the mouse.

[gpui-component](https://github.com/longbridge/gpui-component) (inputs, lists, virtualized
tables, theming) is a likely dependency once real UI work starts. It pins a `gpui-pre`
version, which is why our GPUI pins follow the same snapshots
([ADR 0001](adr/0001-gpui-via-gpui-pre-snapshots.md)).

## Platform

macOS first. GPUI supports Linux and Windows, and nothing above is macOS-only except the
Safari reader and Keychain (both have equivalents elsewhere). Release builds will need an
app bundle, icon, code signing and notarization (`cargo-bundle` or a custom script), plus
auto-update later.
