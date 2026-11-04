# Roadmap

Milestones toward the public alpha. Each milestone leaves the app working and the docs up
to date.

## M0: Foundation (done)

- [x] Cargo workspace, Rust 2024, pinned toolchain
- [x] GPUI app that opens a window titled "Knoa" (Quit via ⌘Q and the app menu)
- [x] rustfmt, taplo, clippy (pedantic, warnings as errors), editorconfig
- [x] CI: format check, clippy, tests on macOS
- [x] Vision, architecture, roadmap and ADR docs

## Research

- [x] Research how people find saved links again (search, browsing, recency) and how
      mymind, Raindrop.io, Capacities, Tana, Eagle and Arc handle navigation, filtering,
      typed fields and AI organizing. Write a short report in `docs/research/` with
      recommendations for Knoa's main screen and search bar
      ([report](research/main-screen.md)).

## M1: App shell and design system

Builds the approved [main screen design](design/main-screen.md).

- [x] Theme tokens (colors, type scale, spacing, radii, animation); a theme system that
      supports several themes, with a solid light default
- [x] Add gpui-component and style it with the theme
      ([ADR 0004](adr/0004-hybrid-ui-components.md))
- [x] Main screen skeleton: floating sidebar with workspace card, places and folders;
      floating search pill with kind chips; content area; floating detail panel; custom
      titlebar
- [x] Masonry card layout, virtualized ([ADR 0007](adr/0007-main-screen-rendering.md))
- [x] Date groups and the date rail
- [x] Up next strip (hideable) and Up next place
- [ ] Optional wallpaper with glass panels from a pre-blurred copy
- [x] View options and appearance popovers
- [x] Core shortcuts, including type-anywhere search
- [ ] Command palette
- [x] Remember window size and position

## M2: Local library

- [x] `knoa-core`: domain types (including the Up next flag), kind definitions, `Query`
- [x] `knoa-store`: SQLite, migrations, queries, tests
      ([ADR 0005](adr/0005-typed-bookmarks-and-folders.md))
- [x] Show bookmarks from the database in the masonry view
- [x] Manual add by URL (⌘N); remove is still to do

## M3: Chromium sync

- [x] Read Chromium-family bookmarks (Chrome, Arc, Brave, Helium, ...) with profile
      discovery
- [x] Watch for changes; idempotent import; mark bookmarks deleted in the browser
- [x] First import stores metadata only
- [x] Onboarding: choose browsers and profiles

## M4: Fetch and extract

- [x] Tokio network runtime and channel bridge to GPUI
      ([ADR 0006](adr/0006-tokio-runtime-for-network.md))
- [x] Page fetch; JSON-LD, Open Graph and oEmbed extraction
- [x] Extractors for popular sites (YouTube, Vimeo, IMDb, GitHub, ...)
- [x] Thumbnail download and disk cache
- [ ] Cards for each kind

## M5: AI organizing

- [x] Proof of concept: one request per bookmark vs. two or three focused requests;
      compare result quality on real bookmarks, then pick one
- [x] OpenRouter API key setup, stored in the Keychain
- [x] Organizing instructions: written during onboarding, editable in settings
- [x] OpenRouter client with structured output
- [x] Durable job queue with retries and a concurrency limit
- [x] Kind, fields, extras, tags and summary; origin tracking, "why" and one-click undo
- [x] Action to process existing bookmarks, with count and cost estimate

## M6: Folders

- [x] One-level folders; a bookmark can be in several
- [x] Description and auto-assign (on by default); removed bookmarks are never re-added
- [ ] "Find bookmarks that belong here" on empty folders; re-scan in every folder (add
      only); hint to re-scan after the description changes

## M7: Search and alpha release

- [x] FTS5 index and instant search
- [ ] Filter chips: kind, site, folder, tag, field comparisons
- [ ] Signed and notarized app bundle; public alpha download

## After the alpha

- [ ] Optional filter sidebar
- [ ] User-defined kinds and fields
- [ ] Firefox and Safari support
- [x] Workspaces: own browsers, instructions, folders and an optional keep guideline
      ([ADR 0011](adr/0011-workspaces-fed-by-browsers.md))
- [ ] Hidden workspaces
- [ ] Re-evaluate Apple's on-device model when a stronger version ships; rejected for now
      ([research](research/apple-intelligence.md))
