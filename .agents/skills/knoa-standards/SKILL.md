---
name: knoa-standards
description: >-
  Knoa's engineering standards for writing code, comments, docs, tests, and pull requests.
  Use for every change in the Knoa repo: adding features, fixing bugs, refactoring,
  writing or deciding on tests, verifying a change in the running app, and opening or
  describing a PR.
---

# Knoa standards

Rust idiom comes from the `rust-best-practices` skill. Use `rust-review` on your own diff
before opening a PR. This skill covers what those don't.

## Rules

1. Readable code beats short code. Small files, small functions, one job each.
2. Fit the change into the codebase. If it only fits with a hack, refactor first.
3. The code is the documentation. Comment only the why. Add no markdown files.
4. Test the core, not every case. Every new test has to earn its place.
5. Prove it works in the running app before you call it done.
6. PR descriptions are short, plain, and human. UI changes get a screenshot.

## Project facts

- GPUI comes from the `gpui-pre` snapshot crates, renamed to `gpui` and `gpui_platform`.
  Its API changes between snapshots, so read the source in
  `~/.cargo/registry/src/*/gpui-pre-<version>/` instead of trusting memory.
- Product intent is in `docs/VISION.md`, the threading model and crate layout in
  `docs/ARCHITECTURE.md`. Read them before structural changes.
- GPL-3.0-or-later. New dependencies need a GPLv3-compatible license (MIT, Apache-2.0,
  BSD, MPL-2.0). Never GPLv2-only or proprietary.

## Workflow

```
- [ ] Read the code the change touches, and its neighbors
- [ ] Refactor first if the change doesn't fit (references/code.md)
- [ ] Implement
- [ ] Add tests only if they pass the bar (references/testing.md)
- [ ] make check
- [ ] Run the app and exercise the change (references/verifying.md)
- [ ] rust-review on the diff, fix what it finds
- [ ] Open the PR (references/pull-requests.md)
```

## References

- [references/code.md](references/code.md): structure, file size, refactoring, comments, docs
- [references/testing.md](references/testing.md): when a test is worth adding, and how to write it
- [references/verifying.md](references/verifying.md): running the app, checking the change, screenshots
- [references/pull-requests.md](references/pull-requests.md): branch, title, description, visuals
