# 0018: Native phone apps, each with its own sync client

- Status: Accepted
- Date: 2026-11-13

## Context

The phone apps were planned as one Expo app sharing the TypeScript sync client in `sdk/`.
Two things changed:

- The phone apps should look and feel like their platform, starting with Liquid Glass on
  iOS. A cross-platform UI layer works against that.
- The phone apps are closed source and live in their own repositories, one per platform.
  This repository is GPL-3.0-or-later and takes contributions under the same license.

A phone is a Cloud mode client only. It has no browser files to import, and iOS won't
reliably run background work, so it doesn't organize: bookmarks saved on a phone are
organized by the server.

Ways to give the iOS app its sync client:

| | Rust crates through UniFFI | Swift client |
| --- | --- | --- |
| Sync logic | Reused from `knoa-store` and `knoa-sync` | Written again, following `sdk/src` (about 2,850 lines) |
| Before starting | Move the sync engine out of the GPUI app crate | Nothing |
| Build | Two toolchains, an XCFramework shipped across repositories | One Swift package |
| License | A closed app links GPL crates | The app only speaks the wire protocol |
| Database | Owned by Rust, with a change callback into Swift | Native observation drives SwiftUI |

The code is written and maintained mostly by AI agents. That makes a third implementation
cheap and a boundary between two languages and two repositories expensive, since every
change and every bug there crosses both.

## Decision

- **Native apps, one repository per platform.** iOS first, Android later in Kotlin with
  Jetpack Compose. No Expo and no shared UI layer.
- **iOS is Swift 6 and SwiftUI,** with iOS 26 as the minimum so Liquid Glass needs no
  availability checks.
- **Each app has its own sync client in its own language,** modelled on the TypeScript
  client in `sdk/src`, not on the desktop store. The phone apps don't link the Rust crates.
- **The protocol and its fixtures are the contract.** `sdk/src/protocol.ts` stays the wire
  contract, and every client runs the scenarios in `sdk/fixtures/` as conformance tests. A
  change to sync behavior lands as a fixture first.
- **Storage on iOS is SQLite through GRDB,** in an App Group container the app and its
  share extension both open. Not SwiftData: the outbox and the pull need explicit
  transactions.
- **The share extension comes first.** It is how bookmarks get into Knoa on a phone.
- **Sign-in is the same Clerk PKCE flow as the Mac app,** with `ASWebAuthenticationSession`
  in place of the loopback listener. The refresh token goes in a Keychain access group the
  extension shares.

## Consequences

- There are three sync clients to keep in step (Rust, TypeScript, Swift), and a fourth with
  Android. The fixtures are what keeps them in step, so behavior no fixture covers can
  drift.
- `sdk/` now serves the web app, the browser extension and the API, and is the reference
  the phone clients are written from.
- The sync engine stays in the Mac app crate. Nothing needs it outside GPUI.
- The phone apps reuse `sdk/` and its fixtures outside the GPL. That only works for code
  whose authors agree to it, so outside contributions to `sdk/` need that agreement, or
  aren't taken.
- Kind definitions still come from `knoa-core` as data. The phone apps read
  `sdk/src/kinds.json`.
