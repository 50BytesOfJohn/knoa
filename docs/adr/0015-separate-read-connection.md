# 0015: One writer and one read-only reader for the database

- Status: Accepted
- Date: 2026-11-10

## Context

The app held one SQLite connection behind one mutex. Every database call waited for the
one before it:

- Loading the library (up to 20,000 bookmarks plus counts, folders, tags and workspaces)
  runs on every change, and up to twice a second while organizing. The organizer's saves
  waited for it.
- A browser import or "organize again" holds the connection for a whole transaction. The
  UI's reads waited for it.

The database is in WAL mode, where a reader on a second connection neither waits for the
writer nor holds it up.

## Decision

- Two connections to the same file: one read-write, one opened read-only. The writer opens
  first and migrates; the reader never migrates.
- The UI's reads (loading the library, AI usage) and the organizer's read of a workspace's
  context go through the reader. Every write, and every read that a write in the same
  critical section depends on, goes through the writer. So do the organizer's queue
  operations.
- Reads that must agree with each other run in one read transaction
  (`Store::snapshot`), since a write can now land between two of them.
- Both connections wait up to 5 seconds when SQLite reports the database busy.
- No pool and no more readers: one is all the UI needs.

## Consequences

- The UI's loads no longer delay saves, and a long import no longer freezes what the UI
  shows.
- A write through the reader fails in SQLite, and the app's reader guard only hands out
  `&Store`, so it doesn't compile either.
- A refresh sees a change only after it has committed. Code that writes and then refreshes
  must wait for the write first, as `Library::change` does.
