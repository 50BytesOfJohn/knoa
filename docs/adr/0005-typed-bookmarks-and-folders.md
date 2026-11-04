# 0005: Typed bookmarks and one-level folders

- Status: Accepted
- Date: 2026-11-04

## Context

Knoa should store what a bookmark is, not just its URL. A saved movie should have its
title, runtime and cast so the library can answer "movies with this actor" or "longer than
two hours". Users also want some manual organization, but not deep folder trees.

## Decision

- **Kinds:** a fixed set (Article, Video, Movie, Recipe, Product, Repository, Book, Image,
  App, Podcast, Place, Other; the last three added after the
  [enrichment research](../research/llm-enrichment.md)), each with typed fields. Kind definitions are data in an embedded file in
  `knoa-core`.
- **Extras:** free-form key/value facts the AI adds when a kind's fields don't cover
  something. Searchable, no typed filters.
- **Tags:** labels, many-to-many with bookmarks.
- `fields` and `extras` are JSON columns on the bookmark. Filters use `json_extract`.
- **Folders:** one level, created only by the user, many-to-many with bookmarks. Each has
  an optional description and an `auto_assign` flag (default on). Folder re-scans only add.
- **Origin:** every AI-set value and folder link records whether it came from the AI or the
  user. AI re-runs never overwrite user values.
- **Exclusions:** (bookmark, folder) pairs the user removed. The AI never adds those
  bookmarks back to those folders.

## Consequences

- Filters on structured facts without asking users to design schemas.
- JSON columns keep the schema small. If field filters become slow, we add generated
  columns or indexes on specific JSON paths.
- No nested folders; kinds, fields and tags provide the structure.
