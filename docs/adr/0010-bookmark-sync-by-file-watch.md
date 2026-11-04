# 0010: Bookmark sync by file watch, with a slow rescan

- Status: Accepted
- Date: 2026-11-04

## Context

New browser bookmarks should show up in Knoa soon after they're saved, without costing
battery or CPU while nothing changes. Options: poll the file on a timer, watch it, or
both.

## Decision

- Watch each chosen profile directory with `notify`, which uses FSEvents on macOS. The
  kernel coalesces events, so a quiet file costs nothing.
- Watch the directory, not the file: Chromium saves `Bookmarks` by writing a temporary
  file and renaming it over the old one, which a watch on the file itself would lose.
- Debounce 1.5 s: one save also rewrites `Bookmarks.bak` and fires several events.
- Rescan every 10 minutes as a backup for missed events (sleep, volume changes).
- Skip the import when Chromium's own `checksum` field is unchanged.

## Consequences

- A new bookmark appears about two seconds after it's saved.
- Rescans are cheap: one small JSON read, then nothing if the checksum matches.
- Chromium also writes `EncryptedBookmarks2`. If it ever drops the plain `Bookmarks` file,
  sync needs the browser's Keychain key, and this decision needs revisiting.
