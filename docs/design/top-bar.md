# Top bar (v2)

- Status: Approved, not built
- Date: 2026-11-24
- Prototype: [top-bar-explorations.html](top-bar-explorations.html), section "Prototype"
  (`#prototype`). Open the file in a browser; the windows are live. The same page is
  published as a private artifact: <https://claude.ai/artifact/9CDT9mTmHETGfjcBQMHL4L>.
  The prototype is a reference for layout and behaviour, not for exact colours.
- Chosen options: 1B (corner capsule), 2D (adaptive row), 3A + 3B (suggestions),
  4A with 4C's overflow (tokens in the pill). Each option is drawn on the same page under
  `#q1`, `#q2`, `#q3` and `#q4`, next to the ones that were not chosen.
- Builds on: round 1, [search-explorations.html](search-explorations.html) ("Trail":
  ↵ locks a word in as a token).
- Background: [research report](../research/top-bar.md)
- Replaces the "Search pill" section and the View options popover of
  [main-screen.md](main-screen.md).

## Principles

- **The pill is only search.** Everything inside it is part of the query. View and
  display controls live elsewhere.
- **What you see is the query.** Every applied filter is a token in the pill, with its
  own ×. There is no second place that shows filters.
- **Browse without typing.** One click on a kind still filters the library, as it does
  today.
- **Only offer what exists.** Every suggestion and refinement shows a count and never
  appears at zero.
- **Calm at rest.** One line under the pill at most. Controls show only while they do
  something (count, clear, Save).
- **Local and instant.** Search, suggestions and refinements never wait for the network
  or the AI.

## Layout

```
                     content area (between sidebar and detail panel)
  ┌───────────────────────────────────────────────────────────────────────────┐
  │                ┌── pill ─────────────────────────────────┐   ┌ capsule ─┐ │
  │                │ ⌕ [in Cooking] [Recipes] [#pasta] quick│    │ ▦ ▤ ☰ │ ⚟│ │
  │                │                          2 found     ✕ │    └──────────┘ │
  │                └────────────────────────────────────────┘                │
  │          Narrow  #italian 1  #quick 1   ·  Clear  ·  Save as folder       │  refine row
  │                                                                           │
  │   Best matches · 2                                                        │
  │   ┌────┐ ┌────┐                                                           │
```

Three parts, top to bottom:

1. **Pill**, centred over the content area, as today.
2. **Corner capsule**, at the top right of the content area, vertically centred on the
   pill.
3. **Refine row**, centred under the pill. One line.

The suggestion list drops down from the pill. The Display popover drops down from the
capsule. Only one of them is open at a time.

`SEARCH_AREA_HEIGHT` stays the pill plus one row of chips plus the gap before the first
card, so the cards don't move when the row's contents change. When the row is empty its
space stays.

## Pill

- Size and look unchanged: 50 px high (`PILL_HEIGHT`), at most 620 px wide
  (`PILL_MAX_WIDTH`), fully rounded, `panel` fill, 1 px `hairline` border, `shadow_md`.
- Focused: the border mixes toward `accent` and gets a soft 3 px `accent` ring.
- Contents, left to right:
  1. Search icon, 17 px, `muted`.
  2. **Tokens** (see [Tokens](#tokens)). They scroll sideways inside the pill if needed;
     the input keeps at least 120 px.
  3. **Input.** Placeholder "Just start typing to search", shown only while there are no
     tokens.
  4. **Count**, "12 found", `text_xs`, `muted`. Only while the query has text or a
     filter, as today.
  5. **Clear button** (×), 30 px round, `muted`, `hover` fill. Only while there is text
     or a token. Clears text and tokens, including the scope token.
- The **view options button is removed** from the pill.

### Tokens

A token is one filter from `Query`, drawn as a small chip inside the pill.

- 26 px high, fully rounded, padding 0 6 0 8 px, 5 px gap: a 12 px icon, the label in
  12.5 px medium, then a 11 px × in `muted` (full `text` on hover). `soft` fill.
- Kinds of token and how they read:

  | Token | Icon | Label | `Query` field | Typed form |
  | --- | --- | --- | --- | --- |
  | Scope | folder or place icon | "in Cooking" | `place` | `in:` (later) |
  | Kind | the kind's icon | "Recipes" | `kind` | `kind:recipe` |
  | Tag | hash | "pasta" | `tags` | `#pasta`, `tag:pasta` |
  | Site | globe | "youtube.com" | `sites` | `site:youtube.com` |
  | Field | lines | "Director Christopher Nolan", "Runtime > 2h" | `fields` | `director:nolan`, `runtime>2h` |
  | Text | search | "“quick”" | `text` | any other word |

- The **scope token** is always first. It has no fill, only a 1 px `hairline` border, so
  it reads as "where" rather than "what". It shows the selected place or folder, as the
  scope chip does today. Everything has no scope token. Removing it selects Everything in
  the sidebar.
- Only one kind can apply (`Query.kind` is an `Option`). Choosing a second kind replaces
  the first token in place.
- Text tokens are joined with the typed text into `Query.text`, so several words still
  mean all of them.
- Clicking a token's label (not the ×) opens a small popover to change its value: the
  same list as the suggestions for that type, with counts. Field tokens also offer the
  comparison (is, more than, less than). Later; for the first build a click only
  focuses the pill.

### Overflow

When the tokens would leave the input less than 120 px, the oldest tokens after the scope
fold into one **"+N" token** (same style, a 11 px chevron instead of the ×). Clicking it
opens a popover listing the folded tokens, each with its ×, and "Clear all" at the end.
The prototype on the explorations page shows this as option 4C.

### Turning text into tokens

- **↵** applies the highlighted suggestion. With "Search for …" highlighted it turns the
  typed text into a text token.
- **Space** after a word with known syntax turns that word into a token straight away:
  `#tag`, `kind:`, `site:`, `tag:` and field comparisons. Uses `Query::apply_filter`, so
  the parser stays the single source of the syntax. A word it rejects stays text.
- **⇥** applies the first filter suggestion (not "Search for …" and not a bookmark).
- The input's text and the tokens always map to one `Query`. Copying the pill's content
  gives the typed form (`kind:recipe #pasta quick`), which parses back to the same tokens.
- Kind names show in the user's language and are stored as slugs, as `translate_search`
  does today.

## Suggestion list

Drops down 6 px below the pill, the pill's width, 16 px radius, `panel` fill, 1 px
`hairline` border, a soft large shadow, 6 px padding. Rows are 32 px high, 9 px radius,
padding 0 10 px, 10 px gap: a 14 px `muted` icon, the label, the count (12 px `faint`,
tabular figures) and, on the highlighted row, a ↵ hint. Group headings are 11 px uppercase
`faint`, 8 px above. The highlighted row has the `hover` fill. At most about 10 rows; it
never scrolls.

A footer line, 11.5 px `faint`, above a `hairline`: "↑↓ move · ↵ apply · ⇥ complete ·
⌫ remove last".

### On an empty focus (3B)

Shown when the pill gets focus with nothing typed and no tokens (click, ⌘F or `/`). Typing
anywhere in the window starts typing straight away and skips it.

1. **Recent**: the last 5 searches in this workspace, drawn as their tokens. ↵ or a click
   restores the whole query.
2. **Saved searches**: later, with Save as folder.
3. **Search by**: one line of syntax hints: `#tag` `site:` `kind:` `runtime>2h`. Not
   selectable.

Recent searches are kept on this Mac per workspace. A search is recorded when the user
opens a bookmark from it or leaves it in place for 3 s. Duplicates move to the top.

### While typing (3A)

In this order, each group only when it has rows:

1. **Search for "…"**: always first, so ↵ does what people expect. Count of results for
   the text.
2. **Kinds** (2 max): kinds whose name starts with the text.
3. **Folders** (2 max): folders whose name or any word in it starts with the text.
   Applying one sets the scope token, the same as clicking the folder in the sidebar.
4. **Tags** (3 max): tags of the current results that start with the text.
5. **Sites** (2 max).
6. **Fields** (2 max): values of typed fields that start with the text, shown as
   "Director: Christopher Nolan".
7. **Bookmarks** (3 max): the top results, a 30 × 24 px thumbnail, the title and the
   site. ↵ on one opens it in the browser, as ↵ on a card does.

Rules:

- Counts are computed within the current tokens, so every row narrows the current view.
  Rows with a count of 0 are left out.
- Values already applied are left out.
- In the label, the typed part is plain and the completion is semibold.
- Starting the text with `#` shows only tags, up to 6.
- All of this comes from the local index on every keystroke, the same as the results.

## Refine row (2D)

The kind chips are replaced by one adaptive row.

- Chips as today: 28 px high, fully rounded, `text_xs` medium, `muted` text on `panel` at
  70 % opacity, full `panel` on hover. Each has its count in `faint`.
- **At rest** (no tokens other than the scope, no text): the kinds in the current view,
  most first, as today. No "All" chip: removing the kind token is "All".
- **While searching or filtered**: a "Narrow" label (11.5 px `faint`), then what splits
  the current results:
  - kinds, while no kind token is applied and the results hold more than one kind (3 max);
  - then tags that some but not all results have, most first.
- Clicking a chip adds it to the pill as a token and removes it from the row.
- **One line only.** Chips that don't fit go behind a "More" chip (no fill), which
  expands the row in place until the query changes.
- At the end, only while at least one token other than the scope is applied:
  **Clear** (removes all tokens and text, keeps the scope) and **Save as folder** (later;
  see [Phases](#phases)). Both are chips with no fill.
- Gallery: starts with tags, since it holds media kinds only (today it hides the row).
  Events: starts with tags as well.
- When the contents change, chips fade in over 120 ms. No fade with Reduce Motion.

## Corner capsule (1B)

- At the top right of the content area: its right edge on the content's right edge, which
  moves left when the detail panel opens. Vertically centred on the pill.
- 36 px high, fully rounded, 3 px padding, `panel` fill, 1 px `hairline` border, a soft
  shadow. On a wallpaper it is glass, like the pill.
- Contents:
  1. **Layout switch**: a segmented control on a `soft` track, three 30 × 26 px segments
     with 14 px icons: Masonry, Grid, List. The current one has the `panel` fill and a
     hairline shadow. Tooltips name the layout. Shown once a second layout exists
     ([Phases](#phases)).
  2. A 1 px × 18 px `hairline` divider (only with the layout switch).
  3. **Display** button: 30 px round, sliders icon 15 px, `muted`; `hover` fill. Tooltip
     "Display". Toggles the Display popover.
- Settings is **not** here. It stays as the gear in the sidebar footer (⌘,), see
  [sidebar.md](sidebar.md).
- Narrow content (less than the pill's width plus the capsule plus 32 px): the layout
  switch hides and its choices move to the top of the Display popover.

### Display popover

Replaces the View options popover (`Overlay::ViewOptions`).

- Drops down 8 px below the capsule, right-aligned with it, 236 px wide, 14 px radius,
  `panel` fill, `hairline` border, soft shadow, 6 px padding. Rows 34 px, label on the
  left, control on the right.
- Rows, in order:

  | Row | Control | Today |
  | --- | --- | --- |
  | Layout | Masonry, Grid, List | Only when the capsule is too narrow for the switch |
  | Card size | Small, Medium, Large | New |
  | Sort | Newest, Oldest; "Best match" added while searching | New |
  | Group by date | Switch | New |
  | Summaries | Switch | Exists (`show_summaries`) |
  | Up next strip | Switch | Exists (`show_up_next`) |

- The "about" line at the bottom of today's popover moves to Settings.
- Changes apply at once and are kept in `SettingsFile`, as today.
- Closes on Esc, on a click outside and when the pill takes focus.

## Keyboard

Changes and additions to [main-screen.md](main-screen.md#keyboard):

| Key | Where | Action |
| --- | --- | --- |
| Any letter | Anywhere | Focus the pill and type (unchanged) |
| ⌘F, `/` | Anywhere | Focus the pill and open the empty-focus list |
| ↑ ↓ | Suggestion list open | Move the highlight |
| ↓ | List closed | Go to the cards (unchanged) |
| ↵ | Pill | Apply the highlighted suggestion, or open the highlighted bookmark |
| ⇥ | Pill, with text | Apply the first filter suggestion |
| Space | Pill, after a known syntax word | Turn the word into a token |
| ⌫ | Pill, empty input | First press selects the last token (accent border), second removes it |
| Esc | Pill | Clear the text; then close the list; then clear the tokens; then go to the cards |

## Phases

**First build** (works on today's `Query`, no new search engine):

- Pill without the view options button; clear button.
- Tokens for scope, kind, tag, site, field and text; overflow "+N".
- Syntax to token on space; ⌫ select-then-remove; Esc order.
- Suggestion list while typing (3A), with bookmarks.
- Refine row (2D), replacing the kind chips.
- Corner capsule with the Display button; Display popover with Summaries and Up next
  strip.

**Next:**

- Empty-focus list (3B) and recent searches (needs a small per-workspace store).
- Layout switch, when Grid and List exist.
- Card size, Sort and Group by date in Display.
- Editing a token by clicking it.

**Later:**

- Save as folder: a folder that fills itself from the query (mymind's Smart Spaces).
  Needs a stored query on folders and sync support.
- `in:` syntax and excluding words (`-word`).
- Plain-language search that turns into AI-marked tokens (round 1, option E).

## Rejected during exploration

| Idea | Why not |
| --- | --- |
| View options inside the pill (1A, today) | Mixes how the library looks with what it shows; no room for a layout switch |
| One labelled Display menu (1C) | Switching layout takes two clicks; the current layout isn't visible. Kept as the narrow-window fallback |
| Window toolbar across the top (1D) | Undoes the floating panels; shrinks search to a small field |
| Always-on kind row with "All" (2A, today) | Wraps to two lines; shows filters in a second place |
| Kind menu inside the pill (2B) | People don't find kinds; two clicks to filter |
| Row only while searching (2C) | Loses one-click browsing by kind, which is how people find things most |
| Two-pane suggestions with large previews (3C) | Hides the cards being narrowed; turns search into a mode |
| Separate applied-filters bar (4B) | Two places to read the query; typed `#tag` would jump out of the field |
| A second settings gear in the corner | Settings already has the sidebar gear and ⌘, |
