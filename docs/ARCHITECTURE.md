# Architecture

The design for the alpha, and what is built of it. When code changes this, update this
file in the same change. Product decisions are in [VISION](VISION.md).

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
| `knoa` | App binary: views, settings, credentials; services that run sync and the organizing queue | yes |
| `knoa-core` | Domain types (`Workspace`, `Bookmark`, `Kind`, `Field`, `Folder`, `Query`), kind definitions | no |
| `knoa-store` | SQLite schema, migrations, queries, full-text search, the job queue | no |
| `knoa-browser` | Chromium profile discovery, `Bookmarks` parsing, file watching, opening links in a profile | no |
| `knoa-fetch` | Page fetch, metadata and JSON-LD extraction, site extractors, thumbnails | no |
| `knoa-ai` | `LanguageModel` and `Classifier` traits, the OpenRouter backend, the organizer | no |
| `knoa-sync` | Cloud mode's client: Clerk sign-in, the Knoa API and its socket | no |

`sdk/` is the shared `@knoa/sync` TypeScript package for the web app, browser extension,
phone app and API repository. `sdk/src/protocol.ts` is the wire contract mirrored by
Rust; `sdk/fixtures/` supplies the shared protocol examples used by the Rust tests.

## Data model

See [ADR 0005](adr/0005-typed-bookmarks-and-folders.md).

- Workspaces own their bookmarks, folders, instructions, synced browser profiles, the
  profile their links open in, and an optional keep guideline ([ADR 0011](adr/0011-workspaces-fed-by-browsers.md)). URLs are
  unique per workspace.
- Kind definitions (kinds and their typed fields) are data in an embedded file in
  `knoa-core`.
- A bookmark stores its kind, `fields` and `extras`. Fields and extras are JSON columns;
  field filters use SQLite's `json_extract`.
- Tags and folders are many-to-many with bookmarks.
- A workspace can have a tag list: up to 40 tags, each with an optional hint of at most
  60 characters, and a switch that holds the AI to the list. Empty by default.
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

Only distribution builds made with `KNOA_CHANNEL=release` use `Knoa`. All other builds,
including optimized builds, keep their library and file credentials in `Knoa Dev` and
disable self-updates, so development never migrates the installed app's library. In the dev
channel, `KNOA_DATA_DIR` puts support files in `<dir>/support` and caches in `<dir>/cache`,
and `KNOA_SKIP_ONBOARDING=1` marks onboarding done, with no browsers chosen and the default
workspace, so a fresh library opens on the main screen.

The app opens the file twice: one connection writes, and a read-only one serves the UI's
reads, so loading the library and saving the organizer's results don't wait for each
other. Reads that must agree run in one read transaction
([ADR 0015](adr/0015-separate-read-connection.md)).

If startup fails, for example because the database is damaged or was saved by a newer
Knoa, a native alert shows the reason and the app quits when it is dismissed.

## Sync

- Reads Chromium-family bookmark files
  (`~/Library/Application Support/<vendor>/<Profile>/Bookmarks`) for Helium, Chrome,
  Brave, Edge, Vivaldi and Chromium, with profile discovery from `Local State`. Arc is
  left out: its sidebar lives in `StorableSidebar.json`.
- Watches the profile directories with `notify` (FSEvents), debounced 1.5 s, plus a
  rescan every 10 minutes as a backup; both skip files whose checksum is unchanged
  ([ADR 0010](adr/0010-bookmark-sync-by-file-watch.md)).
- One-way, browser → Knoa. Each profile is imported into every workspace that syncs it.
  Bookmarks are keyed on workspace and URL, so re-imports are idempotent. The browser folder path is stored and passed to the AI as a
  hint.
- A bookmark deleted in the browser is marked, not deleted.
- A bookmark deleted in Knoa goes to Trash and stays deleted while the browser keeps
  it; bookmarking it again in the same profile brings it back
  ([ADR 0012](adr/0012-deleting-bookmarks-through-trash.md)).
- Links added by hand go straight into the organizing queue and are always kept.
- The first import stores metadata only. AI processing of those bookmarks starts when the
  user triggers it. Later bookmarks are processed automatically.

## Cloud mode

Signed in to a Knoa account, the library also syncs with the account's copy on the server
(the Knoa API, in its private API repository), which the web app and other devices
share. Signed out, the app is Local mode and none of this runs.

- **Sign-in:** Clerk is the OAuth provider. The app opens the system browser on Clerk's
  authorize page, with PKCE and a redirect to a one-shot listener on
  `http://127.0.0.1:<port>/callback`, on a port the system picks (RFC 8252). The refresh
  token is stored like the AI key ([ADR 0008](adr/0008-credential-storage.md)), and the
  access token is refreshed shortly before it runs out.
- **Ids:** linking a library gives every workspace, folder and bookmark a sync id.
  Bookmark ids are UUID v5 of the URL under the workspace id, so two devices importing the
  same bookmark converge on one row. A second device's workspaces and folders are matched
  to the account's by name.
- **Push:** every store change in a linked library also writes an outbox row with a
  client-made id, in the same transaction, recording only the columns that changed and
  who made them (user or AI). The sync loop sends the outbox after local changes and drops
  each mutation once the server has answered it. A bookmark refused for the plan's limit
  waits on this Mac and is sent again whole once the account has room.
- **Pull:** the server gives every changed row a rising version, and the app pulls what
  came after its cursor, one transaction per page. Columns with a pending local change
  keep the local value. A socket pokes the app when something changed; a pull every few
  minutes covers a missed poke.
- **Device-only data** never syncs: browser profiles, the job queue, page evidence,
  `deleted_urls` and AI usage. Thumbnails go through the server's image store instead of
  the rows.
- **Who organizes:** the account says. With Pro, the server's organizer (the same
  `knoa-core`, `knoa-fetch` and `knoa-ai` crates compiled to WebAssembly by the
  Cloudflare Worker in the private API repository) does it, and this
  Mac hands its queue to the server instead of running it. On the free plan this Mac
  organizes with the user's own key, pushes the results and uploads the thumbnails.
- **Account page** in Settings: sign in and out, the plan and its usage, AI access, and
  how sync is doing. AI access settings are fetched when the page is shown and saved on
  change, separately from library sync. Pro accounts can set a default and workspace
  exceptions for connected MCP clients. Signing out keeps the library and its link, so
  signing in to the same account carries on; signing in to another account asks before
  merging the library into it.

## Enrichment

Each bookmark goes through a durable job queue (retries with backoff, about 4 concurrent
network requests), so quitting the app loses no work. When the provider rejects the key or
the account is out of credits, the queue pauses and tries one bookmark every 5 minutes; it
resumes on its own once that one gets through, or when the user sets a new key.

1. **Fetch** the page with `reqwest` (plain HTTP, with timeout and size limit).
2. **Extract** metadata, in this order:
   - site-specific extractors for popular sites (YouTube, Vimeo, IMDb, GitHub, ...);
   - structured data on the page: schema.org JSON-LD, Open Graph, oEmbed;
   - LLM extraction from the page text when the above is not enough.
3. **Kind**: the page's own kind when it declared one, else one choice question to the
   classifier (OpenRouter Decisions), else a small chat call
   ([ADR 0013](adr/0013-kind-before-fields.md)).
4. **Organize** with one structured chat call for that kind (`json_schema`,
   `strict: true`, the kind's fields as a typed object). Output: summary, fields, extras,
   tags, and whether it fits when the workspace has a guideline. Input: the workspace's
   instructions, guideline and tag list, the extracted evidence and about 4k characters
   of page text. A bookmark that doesn't fit is stored as set aside, with the reason, and
   gets no folders. Values read from the page override the model's. When the workspace
   holds the AI to its tag list, tags not on it are dropped.
5. **Folders** come from a yes/no classifier, one question per auto-assign folder of the
   bookmark's workspace
   (OpenRouter Decisions, with a sentinel question). A provider without a classifier
   picks folders in the chat call instead.
6. **Store** the result with the model name. Without AI, steps 3 to 5 are skipped and the
   page's own metadata is stored.

Models follow the [enrichment research](research/llm-enrichment.md):
`openai/gpt-6-luna` with `anthropic/claude-haiku-5.5` as fallback, and
`perplexity/pplx-decider-v1.2` for kinds and folders.

Each answered AI request's tokens and cost, as far as the provider reports them, are
added to daily totals per model in the database. Requests reported without tokens or a
cost are counted, so the AI settings page can say when a total is incomplete. There is no
per-request log; the provider's own dashboard has that.

Folder "find bookmarks" and re-scan run folder matching over bookmarks that are already
processed. They only add.

"Organize again", for one workspace or all of them, queues every bookmark outside Trash
for a full run, so new folders and changed instructions reach bookmarks organized
before. Like any re-run it keeps user values and only adds folders. Its "folders only"
choice queues organized bookmarks as folders-only jobs: no fetch and no chat call, just
the folder classifier over the stored kind, summary and tags. A provider without a
classifier gives those bookmarks a full run.

AI providers sit behind two traits in `knoa-ai`
([ADR 0009](adr/0009-provider-neutral-ai.md)); OpenRouter is a small hand-written client
([ADR 0003](adr/0003-openrouter-thin-client.md)). The API key is stored in the macOS
Keychain in release-channel builds and in a plain file in dev builds
([ADR 0008](adr/0008-credential-storage.md)).

## Search

- SQLite FTS5 over titles, URLs, summaries, tags, field values, extras and page text, with
  prefix matching and BM25 ranking.
- A `Query` in `knoa-core` combines text with filters (kind, site, folder, tag, field
  comparisons such as `runtime > 2h`). `knoa-store` turns it into SQL. The search bar edits
  this `Query`.

## UI

- Main screen: a floating sidebar (workspace and add-link buttons beside the traffic
  lights, places, folders), a floating search pill with kind chips, masonry cards grouped
  by date with a date rail, and a floating detail panel. The approved layout is in the
  [main screen design](design/main-screen.md); the research behind it is in
  [research](research/main-screen.md).
- Settings: a sheet over the main screen with its own sidebar of app pages (General,
  Appearance, AI) and one page per workspace.
- Masonry, glass panels over a wallpaper and the date rail are Knoa's own elements
  ([ADR 0007](adr/0007-main-screen-rendering.md)).
- Components: [gpui-component](https://github.com/longbridge/gpui-component) for input,
  virtualized lists, scrollbars and popovers, styled with Knoa's theme. Cards, search bar,
  detail view, command palette and animations are Knoa's own
  ([ADR 0004](adr/0004-hybrid-ui-components.md)).
- Keyboard-first: command palette and shortcuts for all main actions.

## Distribution

macOS only. The alpha is distributed as a signed and notarized app bundle.

`scripts/bundle.sh` (`make bundle`) builds an optimized `Knoa Dev` bundle from the binary and
`crates/knoa/bundle/`: the `Info.plist` and `AppIcon.icon`, the Icon Composer source of the
app icon. Xcode's `actool` compiles the icon into `Assets.car`, which macOS 26+ shows in
every icon appearance, and `AppIcon.icns` for older systems. Local bundles are signed ad hoc
and omit Sparkle. `KNOA_DEV=1` selects the debug profile; `scripts/package.sh` explicitly sets `KNOA_CHANNEL=release` for distribution.

## Releases and updates

The `Release` workflow bumps the version, writes `CHANGELOG.md` from commit titles, tags
`main`, and publishes a GitHub release with a universal, Developer ID signed and notarized
`Knoa.dmg`. Installed copies update through Sparkle, embedded in the bundle, which reads its
signed feed from the latest release
([ADR 0014](adr/0014-releases-and-updates.md)).
