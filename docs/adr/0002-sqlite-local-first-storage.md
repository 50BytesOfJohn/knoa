# 0002: SQLite for local storage, FTS5 for search

- Status: Accepted
- Date: 2026-11-04

## Context

Knoa stores a personal library: thousands to tens of thousands of bookmarks with metadata,
tags, folders, AI results and a job queue. It must work offline, start instantly and search
fast.

## Decision

- SQLite through `rusqlite` (`bundled` feature), WAL mode, one database file in the app's
  support directory.
- Versioned, embedded migrations run at startup.
- FTS5 for keyword search.
- Thumbnails are cached as files on disk and referenced from the database.

## Alternatives considered

- **Embedded key-value store (redb, sled):** no relational queries or full-text search.
- **libSQL / Turso:** extra dependency with features we don't need.
- **ORM (Diesel, SeaORM):** slower compile times and more abstraction than this schema
  needs. Plain SQL in a repository module is easier to read and tune.

## Consequences

- One file to back up and inspect (`sqlite3`, TablePlus).
- Database access is synchronous, so it runs on GPUI's background executor, never on the
  UI thread.
