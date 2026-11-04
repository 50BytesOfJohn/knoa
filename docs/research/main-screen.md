# Research: finding saved links, and Knoa's main screen

- Date: 2026-11-04
- Question: how do people find saved links again, and what do comparable apps do for
  navigation, filtering, typed fields and AI organizing? What should Knoa's main screen and
  search bar look like?
- Method: published studies on web revisitation and personal information management (PIM),
  plus the official docs of mymind, Raindrop.io, Capacities, Tana, Eagle and Arc. No user
  testing yet.

## How people find things again

1. **Re-finding is the main job, not a corner case.** Most page visits are revisits
   (58% in Tauscher and Greenberg 1997, 81% in Cockburn and McKenzie 2001), and up to 40%
   of web search queries are attempts to find something seen before (Teevan et al. 2007).
2. **People search the web instead of using their bookmarks.** Re-running a web search is
   often faster than finding the link in bookmarks or history (Kaasten 2001). Browser
   bookmark menus fail at re-finding: they are titles in folders, with nothing else to
   match on. This is the gap Knoa fills.
3. **For their own collections, people prefer to navigate.** In Bergman, Whittaker et al.
   (2008), people estimated that they navigated to files 56–68% of the time and searched
   only 4–15%, using search mainly when they could not remember where something was. Better
   search engines did not change this. (Self-reported, but the result holds across later
   studies.)
4. **People remember fragments, not titles.** They recall what something was ("a recipe",
   "a video"), where it came from ("on YouTube"), roughly when they saved it, what it
   looked like, and a detail inside it ("the one with the actor from ..."). They rarely
   remember the exact title. Thumbnails and recency ordering both improve re-finding in
   history lists (Kaasten 2001).

**What this means for Knoa:** search must match every fragment people remember (kind,
site, time, image, any field or detail), and the main screen must also be a place you can
*browse*: the most recent bookmarks first, recognizable thumbnails, and folders you can go
to. Search alone is not enough, even very good search.

## What comparable apps do

| App | Navigation | Search and filters | Typed data | AI organizing |
| --- | --- | --- | --- | --- |
| **mymind** | No folders. Three places: Everything, Serendipity (resurfacing old items), Spaces | One field: keyword, color, topic, type, time. `#tag` limits to a tag | Media type only (article, image, product, ...) | Automatic tags, no review. Smart Spaces are saved searches that fill themselves |
| **Raindrop.io** | Sidebar with nested collections; three panes (sidebar, list, detail) | Operators (`#tag`, `type:`, `created:>`, `-word`, `"phrase"`); filters listed in the sidebar, only those with matches; full-text and meaning-based search (Pro) | Content type, detected automatically | Optional assistant (Stella) for natural-language search |
| **Capacities** | Sidebar of object types; each type has a dashboard | Queries: type, tags, properties, dates; sort, group, limit; saved as views | Object types with properties | Light |
| **Tana** | Outline; each supertag is a searchable collection shown as a table | Query builder with prefix operators (`AND x y`), field comparisons (`LT`); filter, group, sort toolbar | Supertags with fields | AI fields and commands |
| **Eagle** | Left sidebar (folders, smart folders, tags), center grid, right inspector; Tab hides both panels | Search plus a filter bar for tags, color, shape, type | File properties | Little |
| **Arc** | Sidebar of tabs and folders; the Command Bar (⌘T) is the universal entry | Command Bar searches tabs, history and the web | None | Tidy Tabs groups tabs and names the groups automatically; the names cannot be edited |

Lessons:

- **mymind** shows that a calm, visual grid with one search field and no setup works and
  feels good. Its weak point is the lack of structure: no typed filters, so "movies longer
  than two hours" is impossible.
- **Raindrop** shows the operator syntax power users expect, and the useful rule that a
  filter is only offered when something matches it. Its sidebar of filters and nested
  collections is also where it feels heavy.
- **Capacities and Tana** show what typed fields make possible, and also the cost: query
  builders are powerful and intimidating. Knoa should get the same power from chips and
  suggestions, without a builder.
- **Eagle's** sidebar, grid and inspector layout is the standard for a visual library on
  the desktop, and hiding both panels with one key keeps it calm.
- **Arc's** Tidy Tabs shows how automatic organizing goes wrong: AI-made names the user
  cannot change. Knoa's principle (user-owned folders, visible and undoable AI values)
  is the right answer.
- Everyone with saved searches (mymind Smart Spaces, Capacities queries, Eagle smart
  folders) treats them as places, next to normal folders. Knoa's auto-assign folders are an
  AI version of the same idea.

## Recommendations

### Layout

```
┌──────────────────────────────────────────────────────────────────────────┐
│ ● ● ●   [ Search, or type kind:, site:, #tag ...           ]   ✦ 12  ⊞ ☰ │  titlebar = search bar
├────────────┬───────────────────────────────────────────┬─────────────────┤
│ Library    │ 1,284 bookmarks · Recent ▾                │  [hero image]   │
│  All       │ TODAY                                     │  Title          │
│  Unsorted  │ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐           │  youtube.com    │
│            │ │ img │ │ img │ │ img │ │ img │           │  Video · 1h 12m │
│ Folders    │ └─────┘ └─────┘ └─────┘ └─────┘           │  Summary ✦      │
│  Client X  │ THIS WEEK                                 │  Fields         │
│  Kids      │ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐           │  Tags · Folders │
│  + New     │ │     │ │     │ │     │ │     │           │  Extras         │
└────────────┴───────────────────────────────────────────┴─────────────────┘
   places        library (grid or list)                    detail (inspector)
```

1. **The search bar is the titlebar.** One row: traffic lights, the search field, then an
   AI activity indicator (how many bookmarks are being organized) and the grid/list
   switch. No separate toolbar.
2. **A slim, collapsible sidebar of places**: All, Unsorted (in no folder), and the
   user's folders, with "New folder". It holds places, not filters: clicking a folder
   sets the `in:` chip in the search bar, so it edits the same `Query`. It answers the
   navigation preference (finding 3) and gives folders a home for their description and
   auto-assign settings. The filter sidebar planned after the alpha (kinds, sites, tags
   with counts) stays separate. Hidden with ⌘\ or by narrowing the window.
3. **The library** in the middle:
   - With an empty query: everything, newest first, with time headers (Today, This week,
     October, 2024). Recency is the strongest cue people have (finding 4).
   - With a query: sorted by relevance, with a header line showing the count and a sort
     menu (Relevance, Recently added, Title).
   - **Grid** (default) shows content-aware cards; **List** shows dense rows (favicon,
     title, site, kind, key field, date) for large text-heavy libraries. ⌘1 / ⌘2.
4. **The detail view is an inspector on the right**, like Eagle, not a modal. It opens for
   the selected bookmark (Space toggles it), and arrow keys keep moving through the
   library while it updates. It shows the hero image, title, site, kind and its fields as a
   table, summary, tags, folders, extras, the browser and browser folder it came from, and
   actions (Open, Copy link, Edit). Every AI value has a small ✦ mark; hovering shows why it
   was set and an Undo button. Cards show no AI marks, to stay calm.

### Search bar

- **One input, text and chips together.** Free text stays text. Filters become chips
  inside the field: `Movie`, `youtube.com`, `#noir`, `in Client X`, `runtime > 2h`.
- **Suggestions as you type**, grouped (Kinds, Folders, Tags, Sites, Fields), showing only
  values that have matches (Raindrop's rule). Typing `nolan` offers "Director: Christopher
  Nolan" as a field filter as well as the text match. ↑↓ and ⏎ pick, Tab completes.
- **Typed syntax for power users**, turned into a chip on space: `kind:movie`,
  `site:youtube.com`, `#noir`, `in:"Client X"`, `runtime>2h`, `added:<2025`, `-word`,
  `"exact phrase"`. The syntax is the chips' text form, so copying a query gives a string
  that parses back into the same chips.
- **Chips are editable.** Clicking a chip opens a popover to change its value or
  operator; Backspace at the start of the input removes the last chip.
- **Search is local and instant.** It never waits for the network or an LLM.
  Natural-language search ("that long Nolan movie") can come after the alpha as a
  suggestion that turns into chips, never as the default path.
- Esc clears the query, then closes the detail view. `/` or ⌘F focuses the bar from
  anywhere.

### Cards

- **Most cards start bare.** The first import stores titles and URLs only (VISION), so on
  day one most of a library has no image, kind or summary. The bare card must look good:
  favicon, title, domain, browser folder path, and a background tint derived from the
  domain. This card is what users see first, so design it first.
- Then one card layout per kind, all in the same grid slot so the grid stays even and
  easy to virtualize: Video (16:9 thumbnail, duration badge), Movie and Book (poster or
  cover on a blurred backdrop, year, creator), Recipe (photo, total time), Product (photo,
  price), Repository (owner/name, language, stars), Article (image, source, summary).
- Bookmarks that are being organized show a subtle shimmer, not a spinner; when the AI
  finishes, the card changes in place without moving the grid.

### Keyboard

| Key | Action |
| --- | --- |
| `/`, ⌘F | Focus search |
| ⌘K | Command palette |
| Arrows | Move selection |
| Space | Toggle the detail view |
| ⏎ | Open in the default browser |
| ⌘C | Copy link |
| ⌘1 / ⌘2 | Grid / list |
| ⌘\ | Toggle the sidebar |
| Esc | Clear query, then close detail |

### Later

- **Saved searches** as places in the sidebar, next to folders (mymind Smart Spaces,
  Eagle smart folders).
- **Resurfacing**: a "Rediscover" place showing old bookmarks (mymind's Serendipity).
- Natural-language search that proposes chips.

## Outcome

The sidebar was accepted, and the layout was refined in design explorations. The approved
result is the [main screen design](../design/main-screen.md).

## Sources

- Tauscher and Greenberg (1997), Cockburn and McKenzie (2001), via
  [Obendorf et al., CHI 2007](https://vsis-www.informatik.uni-hamburg.de/getDoc.php/publications/280/chi2007-newformat.pdf)
- [Kaasten, MSc thesis (2001)](https://grouplab.cpsc.ucalgary.ca/grouplab/uploads/Publications/Publications/2001-Kaasten.MScThesis.pdf)
- [Teevan et al., Information re-retrieval: repeat queries in Yahoo's logs (SIGIR 2007)](https://people.csail.mit.edu/teevan/work/publications/papers/sigir07.pdf)
- [Bergman, Whittaker et al., Improved search engines and navigation preference in PIM (TOIS 2008)](https://cir.nii.ac.jp/crid/1360855568725033728)
- mymind: [How mymind works](https://mymind.helpscoutdocs.com/article/68-how-mymind-works),
  [Spaces](https://mymind.com/how-to-get-the-most-out-of-spaces),
  [Organizing](https://mymind.helpscoutdocs.com/article/79-how-can-i-help-organize-my-new-mind)
- Raindrop.io: [Using search](https://help.raindrop.io/using-search),
  [Filters](https://help.raindrop.io/filters),
  [3.0 release](https://www.producthunt.com/p/raindrop-io/raindrop-io-3-0)
- Capacities: [Queries](https://docs.capacities.io/reference/queries),
  [Organizational structures](https://docs.capacities.io/reference/organizational-structures)
- Tana: [Search nodes](https://outliner.tana.inc/docs/search-nodes),
  [Supertags](https://outliner.tana.inc/docs/supertags)
- Eagle: [Interface](https://docs-en.eagle.cool/article/474-interface),
  [Inspector](https://en.eagle.cool/article/477-interface-inspector),
  [Sidebar](https://en.eagle.cool/article/475-interface-sidebar)
- Arc: [Tidy Tabs](https://allthings.how/how-to-organize-today-tabs-in-arc-browser-with-tidy-tabs/),
  [Arc help center](https://resources.arc.net/hc/en-us/articles/19335160678679)
