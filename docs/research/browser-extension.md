# Research: a browser extension that adds to Knoa

- Date: 2026-11-05
- Status: built ([ADR 0019](../adr/0019-browser-extension.md))
- Question: how can a Chrome and Firefox extension add the current page, or a link on it,
  to the local Knoa library, from the toolbar button and the context menu, with no account
  and no server?
- Method: Chrome and MDN docs, the host manifests other Mac apps install (1Password,
  Anytype, Claude), and a working proof of concept on the `browser-extension-poc`
  branch (`crates/knoa-bridge`, `extension/`).

## Recommendation

**Native messaging.** The extension calls `runtime.sendNativeMessage("dev.knoa.bridge", …)`.
The browser starts `knoa-bridge`, a small binary shipped inside Knoa.app, and talks to it
over stdin/stdout. The bridge writes the link into `knoa.db` through `knoa-store`, then
pokes the app so a running Knoa shows the link and starts organizing it right away.

- No account, no port, no token: the browser only starts the host for extension ids
  listed in the host manifest, and only Knoa's own binary is on the other end.
- Works with Knoa closed: the link sits in the job queue and is organized at next launch.
- Same mechanism in every Chromium browser and in Firefox; one extension source for both.
- This is how 1Password and Anytype's web clipper do it on this Mac today.

## Options compared

| | Native messaging | Local HTTP server | `knoa://` URL scheme |
| --- | --- | --- | --- |
| Who can call it | Only listed extension ids | Any web page or local process, unless paired with a secret | Any page, after a "Open Knoa?" prompt |
| Setup | App writes one JSON file per browser | Pick a port; handle conflicts; pairing UI | Register scheme in Info.plist |
| Knoa closed | Works (bridge writes the DB) | Fails | Launches and focuses Knoa |
| Reply to the extension | Yes (added, already there, error, workspaces) | Yes | No |
| Browser friction | None | Chrome's Local Network Access prompts web pages for loopback; extension behavior undocumented | Prompt per site unless "always allow" |
| Firefox | Yes | Yes | Yes |

The URL scheme is still worth adding later as a cheap extra entry point (Shortcuts, Raycast,
a bookmarklet, Safari), but not as the extension's channel.

## How it works (macOS)

**Host manifest.** `dev.knoa.bridge.json`, per user, so no admin rights:

| Browser | Directory under `~/Library/Application Support` |
| --- | --- |
| Chrome (Beta, Canary) | `Google/Chrome/NativeMessagingHosts` (`Google/Chrome Beta/…`) |
| Chromium | `Chromium/NativeMessagingHosts` |
| Brave | `BraveSoftware/Brave-Browser/NativeMessagingHosts` |
| Edge | `Microsoft Edge/NativeMessagingHosts` |
| Vivaldi | `Vivaldi/NativeMessagingHosts` |
| Helium | `net.imput.helium/NativeMessagingHosts` |
| Arc | `Arc/User Data/NativeMessagingHosts` |
| Firefox | `Mozilla/NativeMessagingHosts` |

The Chromium directories are each browser's user data directory, the same ones
`knoa-browser` already knows. Chromium manifests list `allowed_origins`
(`chrome-extension://<id>/`, no wildcards); Firefox's list `allowed_extensions` (the
`browser_specific_settings.gecko.id`). `path` must be absolute, so Knoa should rewrite the
manifests at every launch: that also repairs them after the app is moved.

**Framing.** Each message is UTF-8 JSON after a 32-bit length in native byte order.
Limits: 64 MiB to the host, 1 MB back. `sendNativeMessage` starts one host process per
message and takes its first reply, which fits "add this link" exactly; `connectNative`
keeps one process alive and is only needed for streaming.

**Concurrency.** The bridge and the app both write `knoa.db`. WAL mode already allows it;
the PoC adds a 5 s `busy_timeout` so a write during organizing waits instead of failing.
The bridge is shipped in the same bundle as the app, so their schemas never drift.

**Waking the app.** The bridge sends a datagram to `Knoa/bridge.sock`; the app refreshes
and wakes the organizer. Unix socket paths are limited to 104 bytes on macOS; the real
path is about 56, and a sandbox container path about 93, so it fits, but a very long home
directory would not. If that matters, watch a file with FSEvents instead.

**Extension ids.** The Chrome Web Store assigns the id. A `key` in the dev manifest keeps
the unpacked build's id fixed, so the host manifest can list both. Firefox uses the
`gecko.id` for dev and release alike. One `manifest.json` works for both browsers:
`background.service_worker` for Chrome, `background.scripts` for Firefox (both present;
each browser ignores the other since Chrome 121 and Firefox 121). Chrome warns about the
unknown `browser_specific_settings` key and Firefox about `key`; neither is an error.

## What the extension can do beyond "save this URL"

- **Context menu** on pages and links, with a submenu of workspaces when there is more
  than one (the bridge lists them). The toolbar button and `⌥⇧K` save to the last used one.
- **The page itself.** The extension can send the rendered page (title, selection,
  `document.documentElement.outerHTML`, up to 64 MiB). That covers pages Knoa's `reqwest`
  fetch can't read: logins, paywalls, client-rendered apps. `knoa-fetch` would extract from
  the supplied HTML instead of fetching. Biggest quality win of an extension over sync.
- **No duplicate with sync.** Bookmarks are unique per workspace and URL, so a link added
  from the extension and later bookmarked in a synced profile stays one bookmark.
- **No profile mapping.** The host can't tell which browser profile called it, so the
  workspace has to come from the extension (menu choice, remembered default).

## Other platforms

Knoa is macOS-only, but the mechanism carries over unchanged:

- **Windows:** manifests can live anywhere; a registry key points at them:
  `HKCU\Software\Google\Chrome\NativeMessagingHosts\dev.knoa.bridge` (Edge and other
  Chromium browsers have their own keys; Edge also reads Chrome's) and
  `HKCU\Software\Mozilla\NativeMessagingHosts\dev.knoa.bridge`. `path` may be relative.
  Chrome passes an extra `--parent-window=` argument, and stdin/stdout must be in binary
  mode.
- **Linux:** `~/.config/google-chrome/NativeMessagingHosts`, `~/.config/chromium/…`,
  `~/.config/BraveSoftware/Brave-Browser/…`, `~/.mozilla/native-messaging-hosts`.
  Browsers installed as Snap or Flatpak can't start a host outside their sandbox without
  extra work (portals, wrapper scripts); this is the main known pain point.
- **Safari** (not asked, but the gap on macOS): needs a Safari Web Extension built in Xcode
  inside a Swift app extension. Its native messaging goes to that app extension, not to a
  stdio host. Same JavaScript, different bridge. Later.

## Distribution

- Chrome Web Store: one-time developer fee, review. The same listing installs in Brave,
  Edge, Vivaldi, Arc and Helium.
- Firefox: every release must be signed by AMO, listed or unlisted (self-hosted). New
  submissions must declare `data_collection_permissions` (`none` here).
- Knoa must stay outside the Mac App Store sandbox, or the app can't write into other
  browsers' support directories. Developer ID distribution, as planned, is fine.

## What was built

The proof of concept on `browser-extension-poc` became the shipped design, recorded in
[ADR 0019](../adr/0019-browser-extension.md):

- `crates/knoa-bridge`, in `Knoa.app/Contents/MacOS` and signed with the app. Requests
  `{"type":"hello"}` (version and workspaces) and `{"type":"save","url",…}` with an
  optional `title`, `workspace` and page `snapshot`; replies `{"ok":…}`. URLs go through
  `parse_link`, so only `http(s)`. It reads messages until the pipe closes, so both
  `sendNativeMessage` and `connectNative` work. The library location comes from
  `knoa-paths`, the same code the app uses.
- The app writes the host manifests at every launch, only where the content changed:
  `dev.knoa.bridge` for release builds, `dev.knoa.bridge_dev` for Knoa Dev. A plain
  `cargo run` has no bundle and registers nothing.
- The extension sends a cut-down snapshot of the page; the organizer reads it with
  `Fetcher::fetch_snapshot` instead of fetching, and it is deleted with the job.
- The extension itself lives in its own repository and also has a Cloud mode that saves
  through the Knoa API.

The socket path limit above is real in development: a long `KNOA_DATA_DIR` makes the bind
fail, and the app logs that it isn't listening. Links still land and are organized at the
next launch.
