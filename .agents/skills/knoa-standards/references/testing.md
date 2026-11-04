# Testing

Few tests, each one worth keeping.

## Before adding a test, answer

1. Is this core logic: data model, import and dedupe, storage, search, parsing?
2. Would a bug here be silent, or reach the user's data?
3. Does an existing test already cover it?
4. Would this test still be right after a refactor that keeps behavior?

Add it only for yes, yes, no, yes.

## Don't test

- UI layout and rendering. Check those in the running app.
- Thin glue, getters, constructors, and `From` impls.
- Every edge case. Pick the one that would actually break.
- Library behavior (GPUI, SQLite, serde).

## How

- Test behavior through the public API, not private functions.
- One behavior per test. Name it after that behavior: `duplicate_url_merges_folders`.
- Real inputs over mocks. Use an in-memory SQLite database, not a fake store.
- Unit tests live in a `#[cfg(test)] mod tests` at the bottom of the file. Integration
  tests go in `tests/` only when they need several crates together.
- Delete a test that keeps breaking on refactors without catching bugs.
