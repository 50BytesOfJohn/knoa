# 0012: Deleting bookmarks goes through Trash; no archive

- Status: Accepted
- Date: 2026-11-05

## Context

Bookmarks could be added but not removed. Two things shape how removing should work:

- Sync is one-way. A bookmark deleted in Knoa is usually still in the browser, and the
  next import would bring it straight back.
- Read-later apps (Pocket, Readwise Reader, Instapaper) have both Archive and Delete,
  because their main list is an inbox to clear. Library apps (Raindrop.io, Apple Notes,
  Finder) have Delete with a Trash that empties itself after about 30 days.

Knoa is a library, not an inbox. Up next is the queue, and taking something off Up next
is already the "done" step. An archive would be a third hidden state next to Set aside
and removed from the browser, and its only difference from the library would be that
Everything leaves it out. That hides things the user may still want to find.

## Decision

- One way to remove a bookmark: Delete (⌫, or the trash button in the detail panel). It
  moves the bookmark to Trash and takes it off the organizing queue. No archive.
- Trash is a place, shown in the sidebar only while it holds something. Every other
  place, search and the counts leave trashed bookmarks out.
- In Trash a bookmark can be restored or deleted now; Empty Trash asks for a second
  click. Bookmarks are deleted for good 30 days after they were trashed, when Knoa starts.
- A deleted bookmark stays deleted while the browser keeps it. Trashed rows stay in the
  table; for one deleted for good, `deleted_urls` keeps its workspace, URL, profile and
  bookmark id until the browser drops it.
- A deleted bookmark comes back when the user adds the URL by hand, or bookmarks it again
  in the same browser profile (a bookmark id Knoa hasn't seen for it).
- Deleting in Knoa never touches the browser's bookmarks.

## Consequences

- Deleting by accident is undone from Trash for 30 days.
- A URL bookmarked twice in one profile, or in two profiles feeding one workspace, can't
  tell "bookmarked again" from "still there", so it stays deleted until added by hand.
- Thumbnails are named by image URL and shared between bookmarks, so they aren't deleted
  with a bookmark. At launch, Knoa deletes cached thumbnails no bookmark points to, which
  also covers deleted workspaces and replaced images.
