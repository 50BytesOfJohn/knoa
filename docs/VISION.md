# Vision

## The idea

Bookmarking is already a habit: one click on the browser's star. What's broken is
everything after. Bookmarks pile up in one flat folder, nobody organizes them, and they're
never found again.

Knoa keeps the habit and fixes the rest:

1. **Capture stays in the browser.** You keep clicking the bookmark icon. Knoa syncs those
   bookmarks automatically; there is no new capture flow to learn.
2. **AI does the organizing.** Each new bookmark is fetched, read and understood by an LLM,
   then categorized, tagged, summarized and filed.
3. **You organize by describing, not by dragging.** During onboarding you write, in plain
   language, how you want things organized ("keep work and personal apart", "group recipes
   by cuisine", "anything about Rust goes under Programming"). Those instructions are part of
   every organizing request, and you can edit them whenever you like.
4. **Browsing happens in Knoa.** A fast native app built for looking things up and revisiting
   them, rather than a list of titles.

## UX principles

- **Minimal on the surface, powerful underneath.** Great defaults that most people never
  change. Advanced features exist but stay out of the way until needed.
- **Content-aware presentation.** A video shows its thumbnail, an image shows the image, an
  article shows a preview card with title, source, summary and reading time. Appearance
  settings can be tuned per content type.
- **Fast and calm.** Native GPU rendering, instant search, no spinners for things that are
  already local. AI work happens in the background and never blocks browsing.
- **Trustworthy automation.** Every AI decision is visible and reversible: why it was filed
  there, which tags were added, a one-click undo. The user's corrections teach the
  organizing instructions.
- **Local-first and private.** The library lives on the user's machine. The only things
  that leave it are LLM requests to the provider the user configured, with their own key.

## Content types

| Type | Preview |
| --- | --- |
| Video (YouTube, Vimeo, ...) | Thumbnail, duration, channel |
| Image | The image itself |
| Article / blog post | Open Graph image, title, site, AI summary, reading time |
| Repository / docs | Project name, description, language/stars where available |
| Product, tool, other | Favicon + Open Graph card |

## AI providers

- **OpenRouter first and only, initially.** One API key gives access to many models;
  sensible default model, changeable in advanced settings.
- **Later, maybe:** use models through an existing Claude or ChatGPT/Codex subscription,
  for example by driving the locally installed `claude` or `codex` CLI in headless mode.
  Before building this, check each provider's terms on using subscriptions from
  third-party apps. Parked until OpenRouter is solid.

## Parked ideas (later)

These came up early and are worth doing, but not before the core is good:

- **Archive.** A quiet place for things that are done or no longer interesting, kept out of
  the main views but still searchable.
- **Relevance check.** Periodically re-evaluate saved items: is this article still accurate,
  still worth reading, has the link died, is a newer version out?
- **Automatic mark/archive.** Suggest (later: automatically do) archiving of items that are
  too old or no longer relevant, governed by the user's instructions.
- **Semantic search / RAG.** "That article about database indexing I saved last spring", and
  questions answered from the library's own contents. See
  [ARCHITECTURE](ARCHITECTURE.md#search).
- **Read-later states.** Unread / reading / done, with reading-time estimates.
- **Dead-link handling.** Detect broken links and offer an archived copy.
