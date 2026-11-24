# 0020: One language per user, kept by the account, Fluent on the Mac

- Status: Accepted
- Date: 2026-11-20

## Context

Knoa spoke English only: the Mac app, the web app, the extension and everything the AI
wrote. Adding Polish, German, Spanish and French touches every client's text, the dates
and numbers they show, and the organizer's prompt. A user with several clients expects
them to agree, and the AI's summaries and tags should match the UI.

## Decision

- **One language per user** (`knoa_core::Language`: `en`, `pl`, `de`, `es`, `fr`, as
  bare codes). It drives every client's UI and what the AI writes. Existing users keep
  English; a new Mac starts in the system language when Knoa speaks it.
- **The account is the source of truth once signed in.** `Account.language` is `null`
  until a client sets it with `PUT /v1/me/language`; the `account` socket message tells
  other devices. The Mac adopts the account's language, gives the account its own when
  it has none, and sends a change made here. A pick made here stays marked unsent in
  settings until the account has it, so a failed request, being offline or an account
  read already on its way can't bring the older language back. Signed out, the
  language lives in the Mac's settings, and the bridge hands it to the extension.
- **Mac app: Fluent through `i18n-embed` and `i18n-embed-fl`.** One `.ftl` file per
  language, embedded with `rust-embed`. `fl!` checks message IDs against the English
  file at compile time, and tests check that every language has exactly the English
  messages, variables and terms. Fluent's selectors cover Polish plurals (one, few,
  many), and its terms keep one word per product concept in each language.
  The language switches at run time with no restart.
- **Dates, numbers and lists come from macOS's own formatters** (`NSDateFormatter` with
  skeletons, `NSNumberFormatter`, `NSListFormatter`) in the app's language with the
  Mac's region, never from hand-written month names.
- **Web app: Paraglide JS** with the locale read from the account, in the web
  repository. The extension uses a small typed dictionary per language.
- **The AI writes in the user's language.** The prompt names it in English last among
  the rules, so each language keeps one cacheable prefix. Names and titles of works stay
  as written, and listed tags keep their spelling.

## Consequences

- Every user-facing string in the Mac app goes through `t!`. A missing or mistyped
  message ID fails the build, a missing translation fails the tests.
- Four new dependencies in the `knoa` crate (`i18n-embed`, `i18n-embed-fl`,
  `fluent-bundle`, `rust-embed`), all MIT or Apache-2.0.
- Summaries and tags already written stay in their old language; organizing again
  rewrites them. When the cloud organizes, that waits until the account has the new
  language.
- gpui-component's own strings fall back to English outside `en`.
- A language this build doesn't know reads as unset in an account and as English in
  settings (`unknown_as_default`), rather than making either unreadable.
