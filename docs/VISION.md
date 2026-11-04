# Vision

## What Knoa is

A macOS app that turns browser bookmarks into an organized, searchable library.

1. **Capture stays in the browser.** You bookmark as usual. Knoa reads the browser's
   bookmarks and picks up new ones automatically.
2. **Organizing is automatic.** For each new bookmark, Knoa fetches the page and uses an LLM
   to work out what it is, its key facts, tags, a short summary and which of your folders it
   belongs in. There is no review step.
3. **You steer with instructions.** You describe in plain language how you want things
   organized, globally and per folder. Every organizing request includes these
   instructions.
4. **You find things in Knoa.** Search and filters over everything Knoa knows about each
   bookmark.

## Audience

The first release is a public alpha, built mainly for the author's own use. Feedback from
early users is welcome. Rough edges are acceptable; poor design is not. UI and design
quality are a priority from the first milestone.

## Principles

- **Simple by default.** Good defaults, with no setup required. Advanced filtering is
  available but out of the way.
- **The library is visible at once, search is one keystroke away.** The main screen opens
  on the library, newest first, with a sidebar of places and folders and a floating
  search bar. Typing anywhere searches. Text search and filters (kind, site, folder, tag,
  field values) share the same bar; filters show as chips. See the
  [main screen design](design/main-screen.md).
- **Personal look.** Several themes, not only light and dark. An optional wallpaper with
  translucent panels; a solid look by default.
- **Content-aware cards.** Each kind of bookmark is presented in its own way: a video shows
  its thumbnail and duration, a movie its poster, year and cast, an article its image,
  source and summary.
- **Fast.** Instant search, smooth scrolling, no waiting on local data. Network and AI work
  run in the background and never block the UI.
- **Visible, reversible automation.** Every value the AI sets is marked as AI-made, shows
  why it was set, and can be undone in one click. User edits always win over the AI.
- **Local and private.** The library is stored on the user's machine. Only page fetches
  and LLM requests (to OpenRouter, with the user's own key) leave it.

## Library model

| Concept | Purpose | Set by | Example |
| --- | --- | --- | --- |
| **Kind** + fields | What the bookmark is | AI and page metadata | Movie: title, year, runtime, director, cast |
| **Extras** | Other useful facts | AI | `awards: Oscar Best Picture` |
| **Tags** | Labels | AI and user | `noir`, `to-rewatch` |
| **Folders** | Personal organization | User; AI can assign | "Client X", "Watch with kids" |
| **Up next** | Queue of things to read or watch | User only | An essay, a film for Friday |

- **Kinds** are a fixed set: Article, Video, Movie, Recipe, Product, Repository, Book,
  Image, App, Podcast, Place, Other. Each kind has typed fields, which makes filters like "movies with this
  actor" or "videos longer than an hour" possible.
- **Extras** hold facts the kind's fields don't cover. They are shown and searchable, but
  have no typed filters.
- **Folders** are one level deep and created only by the user. A bookmark can be in
  several folders. Each folder has:
  - an optional description of what belongs in it, which the AI uses for matching;
  - an auto-assign switch, on by default, that lets the AI add new bookmarks to it;
  - a "find bookmarks that belong here" action (offered when the folder is empty) and a
    re-scan action. Both only add bookmarks. Bookmarks that no longer match are marked,
    not removed.
- A bookmark the user removes from a folder is never added back to it by the AI.

## Sync

- New browser bookmarks are imported and organized automatically.
- The first import only reads titles, URLs and browser folders. It is instant and free,
  and the bookmarks are immediately searchable. Running the AI over these existing
  bookmarks is a separate action that shows the count and estimated cost first.
- Deleting a bookmark in the browser does not delete it from Knoa; Knoa marks it as removed
  from the browser.

## After the alpha

Planned, but not part of the alpha:

- An optional filter sidebar, hidden by default, showing kinds, sites, folders, tags and
  fields with counts. It edits the same query as the search bar.
- User-defined kinds and fields.
- Firefox and Safari support.
- Workspaces: separate libraries (e.g. Work and Personal) with their own folders and
  instructions, including hidden workspaces.
