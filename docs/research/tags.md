# Research: tags

- Date: 2026-11-04
- Question: what are tags for in Knoa, and how does Knoa keep an AI-grown tag vocabulary useful
  as the library grows to thousands of bookmarks?
- Method: desk research (comparable apps, folksonomy and controlled-vocabulary literature), then a
  throwaway Python PoC. It ran on the 31 real Helium bookmarks followed by 360 synthetic bookmarks
  with known topics (391 in total, processed in order, feeding each result back the way the app
  would). Six strategies were compared. **$0.33 spent** in about 4,400 calls, all logged with
  their cost. Two strategies stopped at 50 bookmarks when the budget ran out (see §6).
- Builds on [LLM enrichment research](llm-enrichment.md) §3.

## Decision (2026-11-04)

Keep the library model to three concepts: **kind** (with its typed fields), **folders** (the
user's own, flexible categories) and **tags**. Tags stay simple:

- **No predefined tags.** The library starts empty; tags grow from what the AI and the user add.
- **The AI adds tags in the existing enrichment call.** It uses the simplest, most common English
  word, **singular** ("cooking", not "culinary-arts"; "movie", not "films"). One system-prompt
  rule; the model is trusted to follow it.
- **At most 30 tags per bookmark**, as a guard against AI spam. Prompts usually produce fewer.
- **No tags only when nothing useful was loaded.** If the fetch returns any usable information
  (title, description, page text, metadata), the AI tags from it.
- **The user can edit, rename, remove and add tags** on any bookmark. User changes always win.
- Code only **normalizes** each tag (lowercase, trim, hyphens, deduplicate). Existing tags are not
  sent to the model, and there is no embedding merge, Decisions check, hierarchy or gardening
  job in the alpha.

The research below explains the trade-offs. §6 lists what was considered and deferred, and what
to watch for once real libraries grow.

## 1. What tags are for

### Tags next to Knoa's other concepts

| Concept | Answers | Example | Who sets it |
| --- | --- | --- | --- |
| Kind | What is it? | Movie, Recipe, Repository | AI + metadata |
| Fields | Facts about it, including **named entities** | director, cast, repo language, site, author | AI + metadata |
| Extras | Other facts | `awards: Oscar Best Picture` | AI |
| **Tags** | **What is it about?** (topic, genre, place, diet) | `film-noir`, `kitesurfing`, `rust`, `japan` | AI and user |
| Folders | Why did *I* save it? (project, purpose, audience) | "mynth", "Watch with kids" | User; AI assigns |
| Up next | What do I do with it next? | queue | User only |

Tags are the only facet that cuts across kinds: one `japan` filter returns the travel article, the
film and the ramen recipe. Entities stay out of tags because they already have typed homes. A
filter on `director = Lynch` or `site:github.com` is more precise than an `#lynch` tag, and
entity tags are what blew up the earlier PoC's vocabulary.

User tags are different. People also tag for themselves ("to-rewatch", "gift-ideas", "client-x").
Golder & Huberman call these *self-reference* and *task-organizing* tags. Knoa allows any user
tag, and the AI never creates this kind.

### Comparable apps

| App | Tags | Lesson for Knoa |
| --- | --- | --- |
| [mymind](https://mymind.helpscoutdocs.com/article/51-how-to-use-tagging-in-mymind) | AI tags on every item (objects, colors, topics), plus manual tags; no folders; [Smart Spaces](https://mymind.helpscoutdocs.com/article/79-how-can-i-help-organize-my-new-mind) are saved searches built from tags or queries | AI tags work as a search aid even when they are not perfect. Mark them as AI-made. Reviews say the AI is weaker on niche topics |
| [Raindrop.io](https://help.raindrop.io/tags) | Free tags plus nested collections; merge in the tag sidebar; AI suggestions. Users [complain](https://better.raindrop.io/bugs/p/local-ai-suggests-tags-that-i-havent-used) that the AI suggests new tags instead of their existing ones, and [ask](https://better.raindrop.io/feature-requests/p/tag-refactoring-tool) for a tag refactoring tool | Prefer existing tags; ship merge and rename from day one |
| Pinboard | Free-form, space-separated; private tags start with "."; [bundles](https://blog.pinboard.in/2012/08/every_day_i_m_bundlin/) group tags (OR); the API's `rename` also folds one tag into another | Rename = merge with an alias is the classic tool for power users |
| [Capacities](https://docs.capacities.io/reference/tags) | Object types say what a thing is; tags group *across* types; [specific entities become objects](https://docs.capacities.io/tutorials/tags-vs-collections), not tags | The same split Knoa makes: kinds/fields for what and who, tags for themes |
| [Tana](https://tana.inc/docs/supertags) | Supertags are types with fields ("is a" test); plain tags are keywords | Knoa's kinds are Tana's supertags; Knoa's tags are the plain keywords |
| Notion | A multi-select property per database. [AI autofill](https://www.notion.com/help/autofill) reuses existing options, and creating new ones is an opt-in switch | A switch or limit on how many new tags the AI may create is a known pattern |
| Karakeep (OSS, AI bookmarking) | LLM tags with a known tag-explosion problem; the maintainers [planned](https://www.github.com/MohamedBassem/hoarder-app/issues/111) to pass only relevant existing tags to the model | Vocabulary in the prompt is the main control; a shortlist is needed only at very large sizes |

### Literature

- **Folksonomies** (Mathes 2004; Golder & Huberman 2006, *Usage patterns of collaborative tagging
  systems*, J. Inf. Sci. 32(2)). Tag use follows a power law with a long tail of one-off tags.
  The main problems are synonyms, polysemy, plurals and **basic-level variation**: people tag at
  different levels of specificity ("python" vs "programming"). One user's vocabulary also
  drifts over time. On del.icio.us, a URL's tag proportions became stable after roughly 100
  bookmarks. That came from imitation, which a single-user library lacks, so Knoa has to
  supply consistency itself.
- **Kinds of tags** (Golder & Huberman; Sen et al. 2006, MovieLens): what or who it is about,
  what it is, who owns it, refining categories, qualities, self-reference and task-organizing.
  Sen et al. found users prefer *factual* tags over *subjective* and *personal* ones for
  browsing. Knoa maps "what it is" to kinds, "who" to fields, and "about" to AI tags. The
  personal and task kinds are user tags only.
- **Controlled vocabularies and thesauri** (ANSI/NISO Z39.19; W3C SKOS 2009): one preferred label
  per concept (`prefLabel`), alternative labels that redirect to it (`altLabel`, "USE/UF"), and
  broader/narrower links (`broader`, "BT/NT"). This is the lightweight structure Knoa needs:
  **aliases** plus an optional **broader** link, not a full taxonomy.
- **Tag gardening** (Peters & Weller 2008, *Webology*): weeding (remove), pruning (merge variants),
  seeding (suggest), fertilizing (add relations). This matches the periodic job in §5.
- **Basic-level categories** (Rosch): people browse most naturally at a middle level ("chair", not
  "furniture" or "kitchen chair"). The prompt rule "the level of a bookstore shelf or a
  subreddit" is a usable stand-in.

## 2. Vocabulary policy (as tested; the alpha uses the simpler rules in the Decision section)

| Question | Policy | Evidence |
| --- | --- | --- |
| Topics vs entities | AI tags are topics only (subject, field, genre, place, diet, activity). Product, company, person, site and project names go to fields/extras. A user can override this in the instructions ("tag Rust crates by name") | Raw prompts: 5.6% entity tags, 6.7% wrong. Policy runs: 0–0.4% entity |
| Specificity | "A bookstore shelf or a subreddit." Name the field, not the umbrella ("compilers", not "software-development"); no ingredients or single dishes ("baking", not "roast-chicken") | Without this rule, `software-development` was on most real bookmarks. Even with it, `artificial-intelligence` / `web-design` became anchors (§4) |
| One idea per tag | Places, diets and genres get their own tag: `travel` + `japan`, not `japan-travel` | The model created `japan-travel`, `portugal-travel`, `spain-travel`, `vegetarian-cooking` |
| How many | 0–3 per bookmark, main topics only; **0 when the content is unknown** | Policy runs average 1.6 tags. Raw averaged 3.4 at half the precision |
| New tags | At most 2 new per bookmark while the library has < 60 tags, then at most 1; enforced in code | A cap of 1 from the start dropped valid tags on early bookmarks (`kitesurfing` + `travel`) |
| Format | Lowercase English slug with hyphens; singular unless the plural is usual. The UI may show a prettier label | — |
| User vs AI tags | Origin is stored per (bookmark, tag). User tags are free-form and never changed by the AI. Topical user tags join the vocabulary the AI sees (in effect a seed list) | VISION: "user edits always win" |
| Language | Tags in English, whatever the page language (Polish pages got English tags). Aliases may hold other languages | `typografia` → `typography` matched by embedding |

## 3. Experiments

### Setup

- **Corpus:** the 31 real Helium bookmarks (as fetched: 12 title-only behind login walls or private
  hosts) first, then 360 synthetic bookmarks written by `gpt-6-luna` from a plan with **58 gold
  topics**. The topics are long-tailed (3 to 41 bookmarks each), interests drift over three
  phases (Rust and macOS rise, ads and auth fade), 12% of pages are partial and 6% title-only,
  and the texts mention many entities. Cost $0.02.
- **Tagger:** `openai/gpt-6-luna`, reasoning `none`, strict JSON schema, a tags-only call (in Knoa
  tags ride in the enrichment call; §7 gives the marginal cost).
- **Judges:** `anthropic/claude-haiku-5.5` graded every tag on 151 bookmarks (all 31 real, 120
  sampled synthetic) as correct / weak / wrong / entity. `anthropic/claude-sonnet-5.5` found
  duplicate-concept groups in each final vocabulary. Findability comes from the gold topics:
  for each topic with ≥ 5 bookmarks, take its most-used tag. **Recall** is the share of the
  topic's bookmarks carrying that tag; **purity** is the share of that tag's bookmarks that
  belong to the topic.
- **Strategies:**
  - `raw`: "suggest 1–5 tags, prefer existing", vocabulary shown, slug only.
  - `rules`: the §2 policy, `existing_tags` / `new_tags` split, budget enforced, kind words stripped.
  - `merge`: `rules` + aliases/plurals + auto-merge at cosine ≥ 0.86 (`qwen3-embedding-8b`), as in
    the earlier PoC.
  - `verify`: `rules` + an embedding shortlist (top 3, cosine ≥ 0.72) → Decisions `noul` "same
    topic?" (`pplx-decider-v1.2`, with a sentinel) → merge if p ≥ 0.8.
  - `garden`: `verify` + gardening every 100 bookmarks (`gpt-6.1-terra`, merge/delete).
  - `decide`: vocabulary-free LLM proposals; existing tags picked by Decisions over an
    embedding shortlist (16 + neighbours).
  - `v2`: `verify` + one-idea-per-tag and no-ingredient rules + gardening with split/broader.
  - `decide` and `v2` stopped at 50 bookmarks (budget).

### Results after 391 bookmarks

| | raw | rules | merge | verify | garden |
| --- | --- | --- | --- | --- | --- |
| Vocabulary size | 135 | 97 | 96 | 90 | 92 |
| Tags used once | 21% | 26% | 20% | 19% | **16%** |
| Tags per bookmark | 3.37 | 1.55 | 1.62 | 1.64 | 1.65 |
| **Precision** (judge: correct) | 62% | 83% | **88%** | 85% | 83% |
| Weak / wrong / entity | 26 / 7 / 6% | 16 / 1 / 0.4% | 12 / 0 / 0% | 13 / 2 / 0% | 15 / 2 / 0% |
| Duplicate groups (tags in them) | 8 (18) | 4 (8) | 5 (10) | 4 (9) | 4 (8) |
| Findability recall / purity | 0.85 / **0.50** | 0.79 / 0.80 | 0.79 / 0.81 | 0.78 / 0.77 | 0.78 / **0.83** |
| Good pages left untagged | 0% | 0.6% | 0.6% | 0% | 0.9% |
| Title-only pages left untagged | **3%** | 55% | 58% | 55% | 55% |
| Merges made (auto / gardening) | — | — | 0 | 0 | 0 / 6 |
| $ per 1k bookmarks (tags-only call) | 0.06 | 0.11 | 0.11 | 0.10 | 0.11 |

Each strategy ran once. With ~250 judged tags per strategy the precision standard error is about
±2.5 points, so `rules` / `merge` / `verify` / `garden` are **statistically indistinguishable**
on precision. The clear gaps are raw vs policy and the shape of the vocabulary.

**Vocabulary growth** (tags vs bookmarks, fitted V ∝ n^β):

| | 50 | 100 | 200 | 300 | 391 | β | extrapolated 1k / 10k |
| --- | --- | --- | --- | --- | --- | --- | --- |
| raw | 36 | 57 | 86 | 110 | 135 | 0.63 | ~240 / ~1,000 |
| rules | 30 | 56 | 72 | 87 | 97 | 0.46 | ~150 / ~430 |
| verify | 27 | 50 | 67 | 82 | 90 | 0.42 | ~135 / ~355 |
| garden | 30 | 52 | 69 | 83 | 92 | 0.43 | ~140 / ~375 |

The synthetic corpus has a closed set of 58 topics, so growth is lower than in a real library;
treat these as lower bounds, perhaps ×2 for a real one.

### Real bookmarks (the author's 31)

- `raw` tagged every login wall and localhost page with filler (`productivity`,
  `bookmark-manager`, `developer-tools`) and put `software-development` + `developer-tools` on
  most of them. It is useless as a filter.
- The policy runs left all 12 unknown pages untagged and gave real pages 1–2 tags
  (`document-conversion` for anydoc, `coding-agents` for prime-agent, `web-design` for 23rd and
  Border Beam, `digital-advertising` for Reddit Ads).
- This library is dominated by apps and tools, so topic tags matter less here than kinds,
  sites and folders. The author's project ("mynth") is a folder, not a tag, which supports the
  split in §1.

## 4. What actually goes wrong as the vocabulary grows

1. **Synonyms barely appear when the vocabulary is in the prompt.** Over 391 bookmarks the
   embedding merge (≥ 0.86) never fired, the verified merge never fired (10 candidates, all
   correctly rejected), and gardening found 1–2 merges per run. The model reuses an existing
   tag when it sees one. The 47-bookmark PoC needed merges because its vocabulary was small at
   the start.
2. **The real problem is granularity**, the basic-level problem from the literature. The
   duplicate groups the judge found are almost all broader/narrower overlaps, not spelling
   variants: `artificial-intelligence` vs `language-models` vs `machine-learning`,
   `image-generation` vs `diffusion-models`, `authentication` vs `identity-management`,
   `application-monitoring` vs `distributed-tracing`.
3. **Early anchors.** The first few dozen bookmarks set umbrella tags that later bookmarks keep
   reusing because they "fit": `web-design` (32–39 uses) and `artificial-intelligence` (24) did
   so despite the rule against umbrellas. The first import shapes the vocabulary most.
4. **Compound and over-specific tags:** `japan-travel`, `vegetarian-cooking`, `startup-pricing`,
   `seed-fundraising`, and ingredient tags on recipes (`lentil`, `chicken`, `orzo`, `risotto`).
   The simple gardener even merged `denmark-travel` into `travel-planning`, which loses the
   place. Compound tags need a **split**, not a merge.
5. **Title-only pages:** the "no tags when unknown" rule works (filler went from 11 of 12 real
   walled or private pages to none), but about half of synthetic title-only pages with a clear title stay
   untagged. That is the right trade-off for precision; tags can be added once a later re-fetch
   succeeds.

## 5. Consistency mechanisms compared

### Canonicalizer on user-typed and imported variants

This is where synonyms do appear: user tags, imports from other apps, other languages and
model changes. The test mapped 20 variants that should map to an existing tag and 15 that
should not against the `garden` vocabulary:

| Method | Correct merges (of 20) | Wrong merges | Missed |
| --- | --- | --- | --- |
| Slug + exact + plural only | 1 | 0 | 19 |
| Embedding auto-merge ≥ 0.86 | **16** | **5** | 4 |
| Embedding shortlist + Decisions check (p ≥ 0.8) | 12 | **0** | 8 |

- Auto-merge's five errors all fold a **broad tag into a narrower one**: `security` →
  `application-security`, `monitoring` → `application-monitoring`, `react` →
  `react-components`, `fundraising` → `seed-fundraising`, `film-noir` → `neo-noir`
  (0.87–0.89). Correct merges sit in the same range (`LLMs` → `language-models` 0.88,
  `AI coding agents` → `coding-agents` 0.90). **No threshold separates them.** Short tag labels
  are a weak embedding signal.
- The Decisions check made no wrong merges but missed `LLMs`, `Rust-lang`, `rust programming`
  and `AI coding agents`. On a hand-made 16-pair set it scored 0.92–1.0 on clear synonyms and
  0.0–0.14 on related-but-distinct pairs; Jev 1.14 was less calibrated.
- Plain embedding similarity also misses things: `A/B testing` vs `experimentation` (0.65),
  `pieczenie` (Polish "baking") vs `baking` (0.71).
- Decision: **never auto-merge silently on embeddings alone.** For AI tags use the verified
  check. For user-typed tags show the nearest existing tags as autocomplete suggestions while
  typing, and let the user choose. For imports, propose the merges in the tag gardening view.

### Picking tags through the Decisions API (`decide`, 50 bookmarks)

- The decider's probabilities are well separated (unrelated tags 0.00–0.02, the sentinel at
  0.001–0.006 every time, so the last-question bias is under control). It gives a useful
  confidence per tag.
- It **reinforces umbrella tags.** Asked "is `artificial-intelligence` a main topic?" about any
  AI page, it says yes (0.84–0.98). Asked about `web-design` on any UI page, also yes. It
  answers the question well, but the question doesn't capture "is this the right level".
- Without the vocabulary, the LLM proposes 2–3 new tags per bookmark (`text-to-image`,
  `generative-ai`, `dry-brining`, `roguelikes`), so the new-tag budget throws away most of
  them. Vocabulary growth after 50 bookmarks was the same as `rules` (32 vs 30).
- It is the slowest setup: an embedding call per bookmark, and `qwen3-embedding-8b` had latency
  spikes of 20–199 s.
- **Not recommended for tagging.** Keep it for its strength: one cheap `noul` per (tag,
  bookmark) to *re-check* tags, for example "find bookmarks that should have this tag" after a
  user creates a tag (the folder equivalent is already in VISION). Revisit as the tag-selection
  path only past ~1,500 tags, when the vocabulary no longer fits the prompt.

### Gardening

- A conservative gardener (`gpt-6.1-terra`, reasoning low, vocabulary with counts and two
  example titles per tag) cost **$0.003–0.005 per run** at ~100 tags and made 1–3 sensible
  operations per run: `postgresql` and `desktop-apps` deleted (entity, kind), `sourdough` →
  `baking`. It lowered one-off tags from 19% to 16% and gave the best purity (0.83).
- Merge-only gardening cannot fix compound tags and can lose information
  (`denmark-travel` → `travel-planning`). The gardener needs **split** (`japan-travel` →
  `travel` + `japan`) and **broader** (`language-models` under `artificial-intelligence`)
  operations; these are implemented in the PoC (`v2`) but did not run long enough to measure.
- In the first 50 bookmarks, the `v2` prompt rules (one idea per tag, no ingredients) gave a
  smaller vocabulary (25 tags vs 30 for `rules`).

## 6. Design for the alpha, and what was deferred

### Alpha

1. **Prompt:** one rule in the system prompt, e.g. "Add tags that describe the bookmark, using the
   simplest, most common English words, singular. Add no tags only if no information about the
   page could be loaded." Plus the user's instructions. The system prompt is static, so it stays
   cached.
2. **Code:** lowercase, trim, spaces → hyphens, drop duplicates, cap at 30. Optionally drop
   tags equal to the kind name ("article", "video"), since the kind filter already covers them.
3. **Data:** `bookmark_tags(bookmark_id, tag_id, origin)`. A tag the user removes from a bookmark
   is not added back by the AI (the same exclusion rule as folders).
4. **Tag manager:** rename, merge (rename onto an existing tag) and delete. Pinboard and Raindrop
   show this is the one cleanup tool users expect.
5. **Search:** tags are matched by text search and `#tag` filters like any other term.

Cost is only the output tokens for the tags: about 3–4 tokens per tag, so even at 30 tags it is
under ~$0.06 per 1,000 bookmarks on `gpt-6-luna`.

### What to watch in real use

- **Tag list size.** More tags per bookmark mean a larger vocabulary. The no-rules run (3.4 tags
  per bookmark) reached 135 tags at 391 bookmarks and extrapolates to ~1,000 at 10,000
  bookmarks. With up to 30 per bookmark it will be several times that. Sort tag lists and
  suggestions by use count so the large list stays usable.
- **Filter precision.** In the test, more tags per bookmark meant broader tag filters (purity 0.50
  vs 0.80). If `#tag` filters start returning loosely related items, lower the typical number in
  the prompt; there is no need to change the model.
- **Filler on unknown pages.** Without a "no information, no tags" rule, login walls got
  `productivity` and similar filler. Keep that rule.
- **Variants across bookmarks.** Without the existing tags in the prompt, the model chooses each
  word fresh. For everyday concepts ("movie", "recipe", "kitesurfing") the simplest word is
  unambiguous. For technical topics the PoC saw several plausible names for one idea ("ai",
  "llm", "language-models"). If this shows up in real use, the cheapest fix is to send the most
  used existing tags in the prompt (§4, §5).

### Deferred (tested here, not needed for the alpha)

- Existing tags in the prompt: the main source of consistency in the PoC, but not needed while
  the simple-word rule holds up.
- Embedding merge of near-duplicate tags: catches variants but wrongly merges broad tags into
  narrow ones (§5).
- Decisions API synonym check: no wrong merges, but an extra call and latency per new tag.
- Picking tags through the Decisions API: reinforces broad tags and is slow (§5).
- A gardening job, aliases and broader-tag links: useful if the vocabulary gets messy at scale.

## 7. Proposed changes to VISION and ARCHITECTURE (suggestions only, not applied)

**VISION.md, Library model table:** change the Tags row to

> | **Tags** | Labels for finding things | AI and user | `noir`, `kitesurfing`, `to-rewatch` |

and add a bullet under the table:

> - **Tags** start empty and grow with the library. The AI adds simple, common English words in
>   the singular (at most 30 per bookmark), and none only when nothing about the page could be
>   loaded. You can edit, rename, merge and add tags; a tag you remove is not added back.

**ARCHITECTURE.md, Data model:** replace "Tags and folders are many-to-many with bookmarks" with:

> - Tags and folders are many-to-many with bookmarks. Tag links record their origin (AI or user);
>   removed AI tags are stored as exclusions, as for folders.

**ARCHITECTURE.md, Enrichment step 3:** remove "existing tags" from the inputs, and add to step 4:
"Tags are normalized (lowercase, hyphens, deduplicated, at most 30)."

## Not tested / open

- `decide` and `v2` ran only 50 of 391 bookmarks (budget). The split/broader gardener and the v2
  prompt rules need a full run; v2 looked better early on (25 vs 30 tags at 50).
- One run per strategy, a single tagger model, and a synthetic corpus with a closed set of
  topics. Model swaps (for example the fallback to Haiku) are a likely source of drift and were
  not tested.
- Whether hiding one-off tags and applying gardening automatically feel right in the UI.
- How the user's instructions ("tag by cuisine", "no tags for work stuff") interact with the
  policy.
- Tag cost was measured with a tags-only call. Prompt caching never applied, because the prompt
  stayed under the 1,024-token caching minimum; in the full enrichment call it will.

The PoC scripts and raw outputs are in the session scratchpad, not in the repository.
