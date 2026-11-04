# 0002: SQLite as the local-first store, FTS5 for search

- Status: Accepted (not implemented yet)
- Date: 2026-11-04

## Context

Knoa keeps a personal library: thousands to tens of thousands of bookmarks with metadata,
tags, AI results and a work queue. It must work offline, start instantly, and search fast.
Semantic search may come later.

## Decision

- SQLite through `rusqlite` (`bundled` feature), WAL mode, one database file in the app's
  support directory.
- Versioned, embedded migrations run at startup.
- FTS5 for keyword search from day one.
- When semantic search arrives, embeddings live in the same database via the `sqlite-vec`
  extension rather than a separate vector service.
- Images/thumbnails are cached on disk, referenced from the database.

## Alternatives considered

- **An embedded key-value store (redb, sled)**: no relational queries or full-text search; we
  would rebuild both.
- **libSQL / Turso**: native vector columns and sync are attractive, but the extra
  dependency isn't justified before we need either. Revisit if multi-device sync becomes a
  goal.
- **An ORM (Diesel, SeaORM)**: heavier compile times and abstraction than this schema needs;
  plain SQL in a repository module is easier to read and tune.

## Consequences

- Single-file backup and easy inspection (TablePlus, `sqlite3`).
- Database access is synchronous, so it runs on GPUI's background executor, never on the
  UI thread.
