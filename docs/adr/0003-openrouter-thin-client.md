# 0003: OpenRouter through a thin hand-written client

- Status: Accepted (not implemented yet)
- Date: 2026-11-04

## Context

Knoa needs an LLM to classify, extract and summarize bookmarks, with structured JSON
output. The user brings their own API key. Many models should be available without
provider-specific code.

## Decision

- OpenRouter is the only provider.
- A small client on `reqwest` + `serde` that covers only what we use: Chat Completions with
  `response_format: json_schema` (`strict: true`) and the models list. No third-party SDK.
- The client sits behind a narrow trait so the pipeline can be tested with a fake.
- The API key is stored in the macOS Keychain.

## Consequences

- We implement retries, rate-limit handling and timeouts ourselves; the API surface is
  small and stable.
- Structured-output support varies by model. The default model must support it, and
  requests set `provider.require_parameters: true` so OpenRouter only routes to endpoints
  that do.
