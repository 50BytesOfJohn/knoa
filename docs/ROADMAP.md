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

- [ ] Theme tokens (colors, type scale, spacing, radii, animation); a theme system that
      supports several themes, with a solid light default
- [ ] Add gpui-component and style it with the theme
      ([ADR 0004](adr/0004-hybrid-ui-components.md))
- [ ] Main screen skeleton: floating sidebar with workspace card, places and folders;
      floating search pill with kind chips; content area; floating detail panel; custom
      titlebar
- [ ] Masonry card layout, virtualized ([ADR 0007](adr/0007-main-screen-rendering.md))
- [ ] Date groups and the date rail
- [ ] Up next strip (hideable) and Up next place
- [ ] Optional wallpaper with glass panels from a pre-blurred copy
- [ ] View options and appearance popovers
- [ ] Command palette and core shortcuts, including type-anywhere search
- [ ] Remember window size and position

## M2: Local library

- [ ] `knoa-core`: domain types (including the Up next flag), kind definitions, `Query`
- [ ] `knoa-store`: SQLite, migrations, queries, tests
      ([ADR 0005](adr/0005-typed-bookmarks-and-folders.md))
- [ ] Show bookmarks from the database in the masonry view
- [ ] Manual add and remove, for development

## M3: Chromium sync

- [ ] Read Chromium-family bookmarks (Chrome, Arc, Brave, Helium, ...) with profile
      discovery
- [ ] Watch for changes; idempotent import; mark bookmarks deleted in the browser
- [ ] First import stores metadata only
- [ ] Onboarding: choose browsers and profiles

## M4: Fetch and extract

- [ ] Tokio network runtime and channel bridge to GPUI
      ([ADR 0006](adr/0006-tokio-runtime-for-network.md))
- [ ] Page fetch; JSON-LD, Open Graph and oEmbed extraction
- [ ] Extractors for popular sites (YouTube, Vimeo, IMDb, GitHub, ...)
- [ ] Thumbnail download and disk cache
- [ ] Cards for each kind

## M5: AI organizing

- [ ] Proof of concept: one request per bookmark vs. two or three focused requests;
      compare result quality on real bookmarks, then pick one
- [ ] OpenRouter API key setup, stored in the Keychain
- [ ] Organizing instructions: written during onboarding, editable in settings
- [ ] OpenRouter client with structured output
- [ ] Durable job queue with retries and a concurrency limit
- [ ] Kind, fields, extras, tags and summary; origin tracking, "why" and one-click undo
- [ ] Action to process existing bookmarks, with count and cost estimate

## M6: Folders

- [ ] One-level folders; a bookmark can be in several
- [ ] Description and auto-assign (on by default); removed bookmarks are never re-added
- [ ] "Find bookmarks that belong here" on empty folders; re-scan in every folder (add
      only); hint to re-scan after the description changes

## M7: Search and alpha release

- [ ] FTS5 index and instant search
- [ ] Filter chips: kind, site, folder, tag, field comparisons
- [ ] Signed and notarized app bundle; public alpha download

## After the alpha

- [ ] Optional filter sidebar
- [ ] User-defined kinds and fields
- [ ] Firefox and Safari support
- [ ] Workspaces
- [ ] Re-evaluate Apple's on-device model when a stronger version ships; rejected for now
      ([research](research/apple-intelligence.md))
