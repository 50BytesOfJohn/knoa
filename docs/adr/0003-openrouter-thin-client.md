# 0003: OpenRouter through a thin hand-written client

- Status: Accepted (not implemented yet)
- Date: 2026-11-04

## Context

Knoa needs an LLM to categorize, tag and summarize bookmarks, with structured (JSON) output.
The user brings their own key. Many models should be available without per-provider code.

## Decision

- OpenRouter is the only provider at first.
- A small client in `knoa-ai` on `reqwest` + `serde`, covering just what we use: Chat
  Completions with `response_format: json_schema` (`strict: true`), the models list, and
  later embeddings. No third-party SDK.
- The provider sits behind a narrow trait (`classify`, `summarize`, `embed`) so additional
  providers (Claude/Codex subscriptions via their CLIs, a local model) can be added without
  touching the pipeline.
- The API key is stored in the macOS Keychain.

## Consequences

- We own retries, rate-limit handling and timeouts; worth it for a small, stable surface.
- Structured-output support varies by model; the default model must support it, and the
  request sets `provider.require_parameters: true` so OpenRouter routes only to endpoints
  that do.
