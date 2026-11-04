# 0006: A dedicated tokio runtime for network work

- Status: Accepted (not implemented yet)
- Date: 2026-11-04

## Context

Page fetches and LLM calls use `reqwest`, which needs tokio. GPUI has its own executor. The
options were GPUI's HTTP client abstraction (`gpui-pre-reqwest-client`) or a separate
tokio runtime. The UI must stay responsive while background work runs.

## Decision

- A small multi-threaded tokio runtime (two worker threads) runs all network work.
- Fetch and AI code is plain async Rust with no GPUI dependency. Results are sent to GPUI
  through channels and applied on the main thread.
- Database access stays on GPUI's background executor
  ([ADR 0002](0002-sqlite-local-first-storage.md)).

## Consequences

- Fetch and AI code is tested with `#[tokio::test]`.
- No dependency on GPUI's HTTP abstraction, which can change between snapshots.
- Two executors, connected only by channels. Nothing on the UI thread waits for the network.
