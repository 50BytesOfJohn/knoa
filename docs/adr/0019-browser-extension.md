# 0019: A browser extension with a local and a cloud mode

- Status: Accepted
- Date: 2026-11-17

## Context

Browser sync only sees bookmarks people make in the browser, and Knoa's own page fetch
can't read pages behind a login or a paywall, or pages built by scripts. An extension can
save any page, link or image in a click, and it sees the page as the user does.

Some people use only the Mac app, signed out. Others use the web app and have no Mac
app at all. An extension has to work for both.

## Decision

- **One MV3 extension** for Chrome-family browsers and Firefox, from one source, in its
  own repository. Saving is a context menu, a toolbar popup with a workspace picker and a
  keyboard command.
- **Two modes, picked by signing in or out.** No automatic fallback between them.
  - **Local mode (signed out):** native messaging to `knoa-bridge`, a host bundled in
    `Knoa.app/Contents/MacOS` ([research](../research/browser-extension.md)). It writes
    to the library itself, so it works with Knoa closed, and pokes a running Knoa to
    refresh and organize. The app registers the host with every installed browser at
    each launch: `dev.knoa.bridge`, or `dev.knoa.bridge_dev` for Knoa Dev, so the two
    never share a library. The bridge finds the library through `knoa-paths`, the same
    code as the app.
  - **Cloud mode (signed in):** HTTPS to the Knoa API, `GET /v1/workspaces` and
    `POST /v1/bookmarks` (`sdk/src/protocol.ts`). A Mac linked to the account gets the
    bookmark through sync.
- **Sign-in is Clerk OAuth with PKCE, like the Mac app,** run through
  `identity.launchWebAuthFlow`. The refresh token stays in the extension's local storage.
- **The page snapshot.** "Save page" can send the page as the user saw it, cut down to
  text, metadata and JSON-LD (no scripts, styles, form controls or most attributes), at
  most `MAX_SNAPSHOT_BYTES` (1 MiB). Organizers read it with `Fetcher::fetch_snapshot`
  instead of fetching; site extractors, JSON-LD and the preview image still apply.
- **Privacy guards on the snapshot.** The extension never sends one for links or images,
  from private windows, with the setting off or the site excluded, or when the page looks
  sensitive: a password, card or one-time-code field, a built-in list of banking,
  payment, mail, health and similar sites, or a page not served over HTTPS. Where it is
  stored, it is short-lived:
  - On the Mac it lives in the bookmark's job row, never in the outbox, and goes when the
    job ends or the bookmark is deleted. A link the library already has drops it. When
    the server organizes for the account, the Mac hands over its queue and the snapshot
    goes with the job.
  - In the cloud it is kept only when the server organizes for the account, in R2 under
    `snapshots/`, deleted after organizing and expired after a day.

## Consequences

- The bundle ships a second binary, signed with the app (`scripts/bundle.sh`).
- Knoa writes into every installed browser's support directory, so it can't move into
  the Mac App Store sandbox.
- A save from the extension and the same bookmark from browser sync stay one bookmark:
  bookmarks are unique per workspace and URL.
- In Cloud mode a snapshot reaches the server only for accounts it organizes for; free
  accounts organize on a Mac, which never sees the snapshot.
