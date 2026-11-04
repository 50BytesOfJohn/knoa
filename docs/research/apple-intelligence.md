# Research: Apple Intelligence instead of OpenRouter

- Date: 2026-11-04
- Question: can Knoa do all its LLM work (kind, fields, summary, tags, folders) with Apple's
  Foundation Models framework only, without OpenRouter?
- Method: a throwaway Swift CLI calling `FoundationModels` (guided generation with a
  `DynamicGenerationSchema` built from the same generic JSON schema), driven by the
  [LLM enrichment PoC](llm-enrichment.md) harness: the same 47 bookmarks, cached pages,
  prompts, pre-pass and validation. Compared with the `openai/gpt-6-luna` results from that
  PoC. macOS 27.0.1, Apple M1, model variant "AFM 3 Core". Greedy sampling.

## Decision (2026-11-04)

**Not used for now.** Quality is clearly below the cheap OpenRouter models, it is 4–5× slower
on an M1, and its 4K context does not fit larger libraries. OpenRouter stays the only
provider ([ADR 0003](../adr/0003-openrouter-thin-client.md)). Revisit when Apple ships a
stronger on-device model or a larger context.

## Results

| | gpt-6-luna (OpenRouter) | Apple on-device (AFM 3 Core) |
| --- | --- | --- |
| Kind agrees with luna | — (46/47 in its own checks) | 37/47 |
| Folders, in the same call | baseline | 24/47 identical to luna; always exactly one folder, often wrong (Rust book → "AI & Agents", AirPods → "Watch", a Paul Graham essay → "mynth") |
| Folders, separate call with folder names + "none" | — | 30/47 identical; misses rather than wrong picks; about 2 s extra |
| Folders, one yes/no question per folder | (Decisions API: 46/47) | 18/47; answers "no" almost always, no probabilities to rank |
| Tags | specific (`rust`, `analytics`, `science-fiction`) | vague, median 1 per bookmark (`tool`, `code`, `mind`, `work`) |
| Fields filled | 104 | 123 (more, not better checked) |
| Latency | p50 about 2 s | p50 9.5 s, p90 14.3 s, max 26 s; about 3 h per 1,000 bookmarks |
| Context | large | 4,096 tokens, prompt and answer together |
| Languages | any | 24 locales; **no Polish** |
| Cost / privacy | about $0.17 per 1,000 bookmarks; data leaves the machine | free, offline, private |

Token budget with AFM's tokenizer: system rules 256, 13 kinds 491, 10 folders 236,
4,000 characters of page text about 1,180, plus the schema and the answer. The real
10-folder setup fits (1,600–2,800 input tokens). The 144-folder stress library alone is
about 1,980 tokens, so it needs a shortlist or several calls per bookmark (not finished;
the test was stopped as not worth it).

## Other findings

- **Private Cloud Compute** (`PrivateCloudComputeLanguageModel`, new in macOS 27): 32K
  context and reasoning levels, but a per-user daily quota (with an iCloud+ upsell) and a
  managed entitlement. Requests from the unsigned test CLI failed with
  `ModelManagerError 1046`, so its quality is untested. The daily quota alone rules it out
  for bulk processing of existing bookmarks.
- **On-device embeddings** (`NLEmbedding.sentenceEmbedding`, 512 dimensions, English only):
  too weak to merge tags or pick folders ("cooking"–"recipe" 0.45, "cooking"–"rust" 0.32).
  Maybe usable for rough semantic search.
- The on-device model is rate-limited when the app runs in the background
  (`GenerationError.rateLimited`).
- **Integration:** FoundationModels is Swift-only, so the Rust app would need a small Swift
  bridge (static library with a C ABI). Guided generation with a dynamic schema works and
  keeps kinds as data.

## Possible later uses

- A "local only" mode for private-network or sensitive bookmarks (kind, title and short
  summary from URL and browser title), where nothing may leave the machine.
- Re-run this comparison when a new model variant or a larger context ships; the harness
  only needs the bookmarks and the luna baseline.

## Not tested

- Edge cases (prompt injection, misleading browser folder, junk titles, Polish pages):
  written but not run.
- 144-folder library with an embedding shortlist or chunked folder calls.
- `contentTagging` use case for tags, and Private Cloud Compute quality.
- Performance on newer Apple silicon (M3/M4 should be faster than the M1 used here).

The PoC scripts and raw outputs are in the session scratchpad, not in the repository.
