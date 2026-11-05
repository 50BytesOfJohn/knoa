# 0014: Releases on GitHub, updates through Sparkle

- Status: Accepted
- Date: 2026-11-05

## Context

Knoa ships outside the Mac App Store. People need a download that Gatekeeper opens
without warnings, and installed copies need to update themselves. Releases should be one
action for the maintainer, with notes people can read.

Updaters considered:

| | Sparkle 2 | Own updater in Rust |
| --- | --- | --- |
| UX | The standard macOS update window: release notes, Install, Remind Me Later, Skip | To design and build |
| Security | EdDSA-signed updates and feed, Developer ID check | To build |
| Install | Replaces the app on quit or now, relaunches, handles permissions | To build |
| Cost | Objective-C calls through `objc2`, about 3 MB in the bundle | Several modules, and their edge cases |

## Decision

- **Sparkle 2** (MIT), embedded in `Contents/Frameworks` and loaded at runtime with
  `NSBundle`, so `cargo run` works without it and has no updater. `SPUStandardUpdaterController`
  shows Sparkle's own windows. Settings › General exposes Sparkle's two preferences:
  check automatically (on by default, daily) and install automatically (off by default, so
  Sparkle asks with the release notes). The calls are `unsafe` objc2 messages, kept in
  `services/updates.rs`; the About panel in `menus.rs` is the only other one.
- **The feed is a release asset.** `SUFeedURL` is
  `releases/latest/download/appcast.xml`, so publishing a GitHub release ships it and
  pre-releases reach nobody. Each release's feed lists only itself. The feed and the
  update are signed with an EdDSA key (`SURequireSignedFeed`,
  `SUVerifyUpdateBeforeExtraction`). The private key lives in the maintainer's login
  keychain and in the `release` environment's secrets.
- **One workflow releases.** `Release` (`make release VERSION=x.y.z`) bumps the version,
  writes `CHANGELOG.md` with git-cliff from commit titles, commits and tags on `main`, then
  on macOS builds a universal app, signs it with the Developer ID, notarizes the DMG,
  staples both, writes the feed, and publishes the release with the changelog section as
  notes.
- **Assets:** `Knoa.dmg` (a stable name, so `releases/latest/download/Knoa.dmg` always
  works), `Knoa-<version>.zip` for Sparkle, and `appcast.xml`. `CFBundleVersion` is the
  number of commits on `main`, which only grows; Sparkle compares that.

## Consequences

- Commit titles are the changelog. They stay plain sentences starting with a verb;
  "Add", "Fix" and "Remove" sort into their own sections, and `[skip changelog]` leaves a
  commit out.
- Losing the EdDSA key means installed copies can't verify new updates. It's backed up
  in the keychain item Sparkle's own tools read.
- The Developer ID certificate expires; `setup-release` warns 90 days ahead, and
  `make renew-certificate` makes a new one (Apple's site does the one step API keys
  can't) and updates the release secret. Signing picks the identity that expires last, so
  the old one can stay in the keychain.
- No delta updates: every update downloads the whole app. Deltas need earlier archives at
  release time; revisit when the app gets large.
- No beta channel yet. A pre-release is a GitHub pre-release that people download by hand.
