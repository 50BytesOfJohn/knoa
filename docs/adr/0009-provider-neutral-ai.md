# 0009: Provider-neutral AI behind two traits

- Status: Accepted
- Date: 2026-11-04
- Amends: [0003](0003-openrouter-thin-client.md) ("OpenRouter is the only provider")

## Context

OpenRouter is the only provider for the alpha, but other providers (direct APIs, local
models, Apple's on-device model once it improves) are likely later. Folder matching
uses OpenRouter's Decisions API, which other providers don't have.

## Decision

- `knoa-ai` defines what Knoa needs, not what a provider offers:
  - `LanguageModel`: structured JSON generation from messages and a schema. Required.
  - `Classifier`: probabilities for yes/no questions. Optional.
- The organizer (`knoa_ai::organize`) is provider-neutral: prompt, schema, validation
  and the folder decision rule live there. With a classifier, folders come from it;
  without one, the chat schema gains a `folders` field.
- `Provider` is an enum over configured backends, chosen from `ProviderConfig` in
  settings (kind, chat models in fallback order, optional classifier model). Adding a
  provider is a new variant and a module implementing the traits.
- Keys are stored per provider in the credential store, keyed by the provider's URL.

## Consequences

- A new provider needs no change to the pipeline, the prompt or the UI beyond its name.
- Static dispatch through the enum keeps async traits simple (no boxing).
- The Decisions API is alpha; if it changes, only `openrouter/decisions.rs` changes.
