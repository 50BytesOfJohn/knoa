# 0013: Pick the kind first, then extract its fields

- Status: Accepted
- Date: 2026-11-05
- Amends: [0009](0009-provider-neutral-ai.md) (`Classifier` answers yes/no questions only)

## Context

Organizing used one chat call that saw every kind's fields and returned the kind with
generic name/value fields. Every new kind added about 34 prompt tokens to every call,
and kinds that overlap compete in one prompt. The schema could not be typed per kind,
because a union of per-kind objects is rejected by some providers.

A comparison on 44 labelled public bookmarks, with today's 12 kinds and with 20
plausible future ones added (several overlapping, such as documentation and article):

| | One call | Kind first, then fields |
| --- | --- | --- |
| Kind correct, 12 kinds | 41/44 | 42/44 |
| Kind correct, 32 kinds | 41/44 | 43/44 |
| Prompt tokens, 12 → 32 kinds | 2,033 → 2,719 | about 1,400 for both |
| Cost per 1,000 bookmarks | $0.18 | $0.18–0.19 |
| Median latency | 1.7 s | 2.1 s |

With 32 kinds the single call filed both documentation pages as articles. Wrong kind
picks from the classifier all had a probability under 0.75.

## Decision

- The kind comes first: the page's own kind when it declares one, else one `choice`
  question to the classifier (OpenRouter Decisions), else a small chat call that only
  returns the kind.
- Extraction is one chat call for that kind: its fields as a typed object (each a string
  or null), plus summary, extras, tags and the guideline answer as before.
- `Classifier` gains `choose`: the probability of each option of one question.

## Consequences

- Prompt size and kind accuracy no longer depend on how many kinds there are.
- Extracted fields always belong to the stored kind, including when the page's kind
  overrides the model.
- About 0.4 s more per bookmark; without a classifier, about 1 s and half again the cost.
- The kind's probability is available for showing uncertain picks; it is only logged
  for now.
