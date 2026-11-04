# Code

## Structure

- One module, one concept. Name it after the concept (`import`, `search`), never
  `utils`, `helpers`, `common`, or `misc`.
- Aim for files under 300 lines and functions under 40. Past that, split by
  responsibility, not by line count.
- Split a file into a directory module (`foo.rs` + `foo/`) once it holds more than one
  concept. Keep `foo.rs` as the public face: re-exports and a `//!` line.
- Order items top-down: public API first, private helpers below the code that calls them.
- Prefer a few more lines with clear names over a dense one-liner.
- No speculative code: no traits with one impl, no config nobody sets, no unused
  parameters "for later".

## Fit the codebase

Before writing, find how the codebase already solves the nearest similar problem and do it
the same way.

Stop and refactor first when the change needs any of these:

- a flag or special case threaded through code that otherwise doesn't care
- copying a function to change one line
- reaching into another module's internals
- a wrapper whose only job is to work around an existing API
- `clone()`, `Rc<RefCell<_>>`, or `unwrap()` added to get past the compiler

Then:

1. Name the shape the code needs so the change fits naturally.
2. Refactor to that shape as its own step, with no behavior change. `make check` passes.
3. Make the change on top.

If the refactor is large or changes architecture, propose the plan to the user before
doing it. If it is a significant decision, record an ADR in `docs/adr/`.

## Comments

- Comment the why: a constraint, a non-obvious decision, a workaround with its cause.
- Never comment the what. If code needs that, rename or split it instead.
- `//!` at the top of each module: one line on what it owns.
- `///` on public items when the name and signature don't say it all.
- No commented-out code, no changelog comments, no `TODO` without a reason.

```rust
// Bad: restates the code.
// Quit when there are no windows.
if cx.windows().is_empty() { cx.quit(); }

// Good: says why.
// A single-window app: closing it should end the process.
if cx.windows().is_empty() { cx.quit(); }
```

## Docs

- Don't add markdown files. A reader learns the codebase from the code and its comments.
- The existing `docs/` (vision, architecture, ADRs) stays. Update it in the same change
  when the code makes it wrong.
- Never write a README, guide, or summary file for a change.
