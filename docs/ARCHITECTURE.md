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
| `knoa-fetch` | Page fetch, metadata and JSON-LD extraction, site extractors, thumbnails, image decoding | no |
| `knoa-ai` | `LanguageModel` and `Classifier` traits, the OpenRouter backend, the organizer | no |
| `knoa-sync` | Cloud mode's client: Clerk sign-in, the Knoa API and its socket | no |
| `knoa-paths` | The build's channel (Knoa or Knoa Dev) and where its library lives | no |
| `knoa-bridge` | Native messaging host: the browser extension saves links through it ([ADR 0019](adr/0019-browser-extension.md)) | no |

`sdk/` is the shared `@knoa/sync` TypeScript package for the web app, browser extension
and API repository. `sdk/src/protocol.ts` is the wire contract mirrored by Rust;
`sdk/fixtures/` supplies the shared protocol examples used by the Rust tests. The phone
apps are native, in their own repositories, with sync clients written against the same
contract and fixtures ([ADR 0018](adr/0018-native-phone-apps.md)).

## Data model

See [ADR 0005](adr/0005-typed-bookmarks-and-folders.md) and, for folders,
[ADR 0017](adr/0017-nested-folders-and-emoji-icons.md).

- Workspaces own their bookmarks, folders, instructions, synced browser profiles, the
  profile their links open in, and an optional keep guideline ([ADR 0011](adr/0011-workspaces-fed-by-browsers.md)). URLs are
  unique per workspace.
- Kind definitions (kinds and their typed fields) are data in an embedded file in
  `knoa-core`, copied to `sdk/src/kinds.json` for the web.
- An event's `starts` and `ends` are `datetime` fields: `YYYY-MM-DD` (all day),
  `YYYY-MM-DDTHH:MM` (floating wall time) or that with a UTC offset (an instant).
  `knoa-core`'s `event` module and the SDK's `events.ts` read them without a time-zone
  database; each app places them in the viewer's local time.
- A bookmark stores its kind, `fields` and `extras`. Fields and extras are JSON columns;
  field filters use SQLite's `json_extract`.
- Tags and folders are many-to-many with bookmarks.
- A workspace can have a tag list: up to 40 tags, each with an optional hint of at most
  60 characters, and a switch that holds the AI to the list. Empty by default.
- Every AI-set value and folder link records its origin (AI or user). AI re-runs never
  overwrite user values. The summary records who wrote it too (`summary_origin`): once
  the user edits it in place in the detail panel, the organizer leaves it alone. A title
  the user sets is safe already, since the AI only replaces a URL standing in for one.
- Folders have a name, an optional description, an `auto_assign` flag (default on), a
  color, an icon (a glyph or one emoji) and an optional parent, at most three levels
  deep. `FolderTree` in `knoa-core` (mirrored in `sdk/src/folder-tree.ts`) turns the
  rows into the tree every view reads.
- Folder exclusions store (bookmark, folder) pairs the user removed, so the AI never adds
  them back.

The exact schema is written with `knoa-store`.

## Storage

SQLite via `rusqlite` (`bundled`), WAL mode, embedded versioned migrations. The database is
`~/Library/Application Support/Knoa/knoa.db`. Thumbnails are cached as files in the app's
cache directory. See [ADR 0002](adr/0002-sqlite-local-first-storage.md). For a link
straight to an image file the cached file, and the one uploaded to the server, is a
display copy of at most 1600 pixels, not the original
([ADR 0016](adr/0016-images-seen-from-derived-copies.md)).

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
- The browser extension saves links in Local mode through `knoa-bridge`, which the app
  registers with every installed browser at launch. The bridge writes to the library
  directly, then pokes the app over `bridge.sock` to refresh and organize. A page
  snapshot it sends stays in the job row and replaces the page fetch
  ([ADR 0019](adr/0019-browser-extension.md)).
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
  same bookmark converge on one row. A second device's workspaces are matched to the
  account's by name, and its folders by path of names.
- **Push:** every store change in a linked library also writes an outbox row with a
  client-made id, in the same transaction, recording only the columns that changed and
  who made them (user or AI). The sync loop sends the outbox after local changes and drops
  each mutation once the server has answered it. A bookmark refused for the plan's limit
  waits on this Mac and is sent again whole once the account has room.
- **Pull:** the server gives every changed row a rising version, and the app pulls what
  came after its cursor, one transaction per page. Columns with a pending local change
  keep the local value. A socket pokes the app when something changed; a pull every few
  minutes covers a missed poke.
- **Device-only data** never syncs: browser profiles, the job queue (with the browser
  extension's page snapshots), page evidence, `deleted_urls` and AI usage. Thumbnails go through the server's image store instead of
  the rows.
- **Who organizes:** the account says. With Pro, the server's organizer (the same
  `knoa-core`, `knoa-fetch` and `knoa-ai` crates compiled to WebAssembly by the
  Cloudflare Worker in the private API repository) does it, and this
  Mac hands its queue to the server instead of running it. On the free plan this Mac
  organizes with the user's own key, pushes the results and uploads the thumbnails.
- **Browser extension:** signed in, it saves through the API instead of the bridge, and
  the bookmark reaches this Mac by sync.
- **Language:** one per user, for every client's UI and for what the AI writes
  ([ADR 0020](adr/0020-one-language-per-user.md)). Signed in, it is an account setting
  (`PUT /v1/me/language`, then `account` on the socket) and the account wins: this Mac
  adopts the account's language and pushes its own when the account has none. A change
  made here stays marked unsent in settings until the account has it, so an older
  account read can't undo it. Signed out, it lives in this Mac's settings.
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

1. **Fetch** the page with `reqwest` (plain HTTP, with timeout and size limit: 4 MiB of a
   page, 24 MiB for a link straight to an image file, 8 MiB for a preview image; a cut-off
   image is refused, never stored).
2. **Extract** metadata, in this order:
   - site-specific extractors for popular sites (YouTube, Vimeo, IMDb, GitHub, ...);
   - structured data on the page: schema.org JSON-LD, Open Graph, oEmbed. The most
     specific item decides the kind: an event over the venue and ticket products on its
     page, a recipe or movie over a video;
   - LLM extraction from the page text when the above is not enough.

   A link straight to an image file is the bookmark itself. It is downloaded once and
   decoded within a budget checked from its headers before any pixels are allocated: a
   pixel count (100 MP on the Mac, 64 MP in the cloud organizer's 128 MB Worker) and the
   decoder's estimated peak memory for that format, sampling and bit depth, which must fit
   beside the body actually downloaded (1 GiB beside the largest body on the Mac; in the
   Worker the whole pipeline stays under 100 MB, so 75 to 84 MB depending on the body; the
   arithmetic is in `knoa-fetch`'s `decode/budget.rs`). JPEGs are scaled down while
   decoding, and one that doesn't fit at the display size decodes at a smaller scale and
   gets a smaller copy. Large images are box-reduced before the final filter, and an
   animation decodes to its first frame. Larger files are refused. One decode gives the
   display copy (JPEG, or PNG while it has transparency; a small, still, upright file
   within the display size is kept as it is) and a 1024 pixel JPEG for the AI. Its
   format, resolution, size and animation go into the evidence, and resolution and format
   into the image kind's fields. An image that can't be decoded (AVIF, HEIC, SVG, lossless
   JPEG, over budget) keeps those details and a note why.
3. **Kind**: the page's own kind when it declared one, else one choice question to the
   classifier (OpenRouter Decisions), else a small chat call
   ([ADR 0013](adr/0013-kind-before-fields.md)).
4. **Organize** with one structured chat call for that kind (`json_schema`,
   `strict: true`, the kind's fields as a typed object). Output: summary, fields, extras,
   tags, and whether it fits when the workspace has a guideline. Input: the workspace's
   instructions, guideline and tag list, the extracted evidence and about 4k characters
   of page text. A bookmark that doesn't fit is stored as set aside, with the reason, and
   gets no folders. The summary, tags, a generated title, extras, free-text field values
   and the fit reason are written in the user's language (the Mac's setting for its own
   organizer, the account's for the cloud's); names and titles of works stay as written,
   and listed tags keep their spelling. For an event the prompt also carries the
   organizer's date (the Mac's local date, the cloud's UTC date), so a date without a year resolves to the
   next one. Values read from the page override the model's, except in a field marked
   `translated` in `kinds.json` (a genre, a cuisine) outside English, where the model's
   answer, the page's value translated, wins. When the workspace
   holds the AI to its tag list, tags not on it are dropped. A bookmark whose title is
   still its URL also gets a short title, which replaces the URL under the same rule as
   the page's own title. Some endpoints answer that title empty whatever the schema says;
   then the link's decoded file name, or its site, stands in rather than asking again. For a linked image the model sees the AI copy, inline at high
   detail in the bookmark's message, after a short system message on describing images;
   when every configured model refuses the request with the image (unbilled), the
   bookmark is organized again from the image's details alone. Image bookmarks organized
   before the AI saw pixels are queued once more by a migration, as a backfill each Mac
   drops for a bookmark whose answer from another Mac has arrived.
5. **Folders** come from a yes/no classifier, one question per auto-assign folder of the
   bookmark's workspace, named by its full path with its parents' descriptions as
   context (OpenRouter Decisions, with a sentinel question). Each folder is asked about
   on its own, never only when its parent matched. A provider without a classifier
   picks folders in the chat call instead. Only the most specific AI pick in a branch
   keeps a link: a pick under an AI link replaces it, and a pick above any existing membership is skipped.
   Links the user made are never changed by this.
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
  comparisons such as `runtime > 2h`; dates and date-times compare by their date prefix,
  `starts>2026-12-01`). `knoa-store` turns it into SQL. The search bar edits this `Query`.
- `Place::Events`, like the Gallery, is device-side and not part of the sync protocol: the
  workspace's bookmarks of kind event, soonest `starts` first and undated ones last.

## UI

- Main screen: a floating sidebar (workspace and add-link buttons beside the traffic
  lights, places, folders, a status card for the AI and sync, and the account menu and
  settings in its footer), a floating search pill with kind chips, masonry cards grouped
  by date with a date rail, and a floating detail panel. The approved layout is in the
  [main screen design](design/main-screen.md), the [sidebar design](design/sidebar.md) and the
  [top bar design](design/top-bar.md) (approved, not built);
  the research behind it is in [research](research/main-screen.md).
- The sidebar and the main view are sibling views, each drawn from its own cache, so the
  sidebar's hovers and animations never rebuild the cards. The sidebar redraws for a
  library, organizer or cloud update only when it changes what the sidebar shows, not for
  new search results or another sync round. Animations follow macOS's
  Reduce Motion, which the app hands to GPUI at launch and whenever it changes.
- Places: Everything, Gallery, Events, Up next, Unsorted, Not organized, Set aside and
  Trash.
  The Gallery shows bookmarks of a media kind (today only images) while the workspace has
  one. It is a device-side `Place`, not part of the sync protocol. Its results are laid
  out as pictures only: each tile at the stored image's shape, cropped to a range, in one
  continuous grid with the title and site over the picture on hover or selection.
- Events: a place shown while the workspace has event bookmarks, after the Gallery. It is
  a month calendar with an Upcoming panel (the clicked day, then happening now and
  upcoming by day, then undated) or one list (happening now, upcoming by day under month
  names, undated, then past), as the `events_view` setting on this Mac says; it is never
  synced. Times with an offset are converted to the Mac's time zone with jiff, floating
  times and dates stay as written. The page builds its model from the results once per
  results and minute, so events move on to happening and past, and a new day moves
  Today, without a restart. Months move with ⌘← and ⌘→ and ⌘T goes to today: plain
  letters start a search. List thumbnails follow the card grid's image release.
- Image viewer: a full-window overlay with its own key context that occludes the screen
  beneath, opened with Space on a picture, a double-click on a Gallery tile or the detail
  panel's cover. In the Gallery a click only selects; Show details (Cmd-I), in the
  bookmark menu and the viewer's bar, opens the detail panel. App-wide shortcuts close the
  viewer before they act. It shows the
  stored display copy no larger than its own size and moves through the pictures among
  the current results. It can copy the picture to the clipboard as an image and save it
  through the save panel.
- GPUI keeps every image it decodes, in memory and in the GPU atlas, until told to drop
  it. The card grid records the pictures it built and releases those more than a screen
  beyond the viewport, except the selected bookmark's. A picture whose cached file a fetch
  wrote again is released too, the selected one included, so it is decoded anew.
- Settings: a sheet over the main screen with its own sidebar of app pages (General,
  Appearance, AI) and one page per workspace.
- Language: every string comes from the Fluent files in `crates/knoa/i18n` through `t!`
  (`i18n` module), and dates, numbers and lists from macOS's formatters in that language
  with the Mac's region (`format` module). The picker is in General and in onboarding,
  which starts in the Mac's language when Knoa speaks it. Switching redraws every window
  and rebuilds the menus at once; views re-set the placeholders their inputs keep, and
  a search keeps `kind:` filters as slugs so it means the same in the new language.
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
and omit Sparkle. `knoa-bridge` goes into `Contents/MacOS` beside the app and is signed
before it. `KNOA_DEV=1` selects the debug profile; `scripts/package.sh` explicitly sets `KNOA_CHANNEL=release` for distribution.

## Releases and updates

The `Release` workflow bumps the version, writes `CHANGELOG.md` from commit titles, tags
`main`, and publishes a GitHub release with a universal, Developer ID signed and notarized
`Knoa.dmg`. Installed copies update through Sparkle, embedded in the bundle, which reads its
signed feed from the latest release
([ADR 0014](adr/0014-releases-and-updates.md)).
