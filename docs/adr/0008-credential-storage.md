# 0008: Keychain in signed builds, plain file in dev builds

- Status: Accepted (not implemented yet)
- Date: 2026-11-04

## Context

The OpenRouter API key ([ADR 0003](0003-openrouter-thin-client.md)) has to survive
restarts. The macOS Keychain is the right home for it, but macOS ties a Keychain item's
access to the app's code signature. `cargo build` produces an unsigned or ad-hoc-signed
binary whose signature changes on every rebuild, so in development macOS asks for the
login password on each new build, even after "Always Allow". A build signed with a stable
Developer ID keeps the same identity across updates and is not prompted.

## Decision

- A small `CredentialStore` trait (read, write, delete) with two implementations. The rest
  of the app doesn't know which one is in use.
- **Release builds** use the Keychain through GPUI's `cx.write_credentials`,
  `cx.read_credentials` and `cx.delete_credentials`. No extra dependency (this replaces
  the `keyring` crate mentioned in the architecture doc).
- **Debug builds** (`cfg!(debug_assertions)`) use a plain JSON file,
  `dev-credentials.json`, in the app's support directory, created with `0600`
  permissions.

## Consequences

- No Keychain prompts during `make dev`. Shipped, signed builds store the key encrypted
  and scoped to Knoa.
- A dev build and a release build don't share a stored key; each is entered once.
- In dev builds the key sits on disk unencrypted. It's readable by any process running as
  the user, and can end up in backups. That's acceptable for a developer's own key; the
  file must never be used by release builds.
- A locally built, unsigned release build will still prompt. That's expected.
